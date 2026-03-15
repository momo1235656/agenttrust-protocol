use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::crypto::hashing::{compute_hash, verify_chain, AuditEntry, GENESIS_PREV_HASH};
use crate::error::{AppError, AppResult};

pub async fn record(
    db: &PgPool,
    agent_did: &str,
    transaction_id: &str,
    amount: i64,
    status: &str,
    timestamp: DateTime<Utc>,
) -> AppResult<String> {
    // Get the last audit log entry for this agent
    let last = sqlx::query_as::<_, (i64, String)>(
        "SELECT index_num, hash FROM audit_logs WHERE agent_did = $1 ORDER BY index_num DESC LIMIT 1",
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await?;

    let (index, prev_hash) = match last {
        Some((last_index, last_hash)) => (last_index + 1, last_hash),
        None => (0, GENESIS_PREV_HASH.to_string()),
    };

    // The timestamp string must match the Python isoformat() output
    let timestamp_str = timestamp.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    let entry_hash = compute_hash(index, transaction_id, amount, status, &timestamp_str, &prev_hash);

    sqlx::query(
        r#"
        INSERT INTO audit_logs (index_num, agent_did, transaction_id, amount, status, timestamp, prev_hash, hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(index)
    .bind(agent_did)
    .bind(transaction_id)
    .bind(amount)
    .bind(status)
    .bind(timestamp)
    .bind(&prev_hash)
    .bind(&entry_hash)
    .execute(db)
    .await?;

    Ok(entry_hash)
}

pub async fn get_chain(db: &PgPool, agent_did: &str) -> AppResult<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, String, i64, String, DateTime<Utc>, String, String)>(
        r#"
        SELECT index_num, transaction_id, amount, status, timestamp, prev_hash, hash
        FROM audit_logs
        WHERE agent_did = $1
        ORDER BY index_num ASC
        "#,
    )
    .bind(agent_did)
    .fetch_all(db)
    .await?;

    let mut entries: Vec<AuditEntry> = Vec::new();
    let mut chain_json: Vec<serde_json::Value> = Vec::new();

    for (index_num, transaction_id, amount, status, timestamp, prev_hash, hash) in &rows {
        let timestamp_str = timestamp.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        chain_json.push(serde_json::json!({
            "index": index_num,
            "transaction_id": transaction_id,
            "amount": amount,
            "status": status,
            "timestamp": timestamp_str,
            "prev_hash": prev_hash,
            "hash": hash,
        }));
        entries.push(AuditEntry {
            index: *index_num,
            transaction_id: transaction_id.clone(),
            amount: *amount,
            status: status.clone(),
            timestamp: timestamp_str,
            prev_hash: prev_hash.clone(),
            hash: hash.clone(),
        });
    }

    let chain_valid = if entries.is_empty() {
        true
    } else {
        verify_chain(&entries)
    };

    let total = entries.len();
    let succeeded = entries.iter().filter(|e| e.status == "succeeded").count();
    let success_rate = if total > 0 {
        succeeded as f64 / total as f64
    } else {
        1.0
    };

    Ok(serde_json::json!({
        "agent_did": agent_did,
        "chain": chain_json,
        "chain_valid": chain_valid,
        "total_transactions": total,
        "success_rate": success_rate,
    }))
}

pub async fn verify(db: &PgPool, agent_did: &str) -> AppResult<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, String, i64, String, DateTime<Utc>, String, String)>(
        r#"
        SELECT index_num, transaction_id, amount, status, timestamp, prev_hash, hash
        FROM audit_logs
        WHERE agent_did = $1
        ORDER BY index_num ASC
        "#,
    )
    .bind(agent_did)
    .fetch_all(db)
    .await?;

    let entries: Vec<AuditEntry> = rows
        .iter()
        .map(|(index_num, transaction_id, amount, status, timestamp, prev_hash, hash)| {
            let timestamp_str = timestamp.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
            AuditEntry {
                index: *index_num,
                transaction_id: transaction_id.clone(),
                amount: *amount,
                status: status.clone(),
                timestamp: timestamp_str,
                prev_hash: prev_hash.clone(),
                hash: hash.clone(),
            }
        })
        .collect();

    let chain_valid = if entries.is_empty() {
        true
    } else {
        verify_chain(&entries)
    };

    let verified_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    Ok(serde_json::json!({
        "agent_did": agent_did,
        "chain_valid": chain_valid,
        "total_entries": entries.len(),
        "verified_at": verified_at,
    }))
}
