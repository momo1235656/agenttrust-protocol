use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub async fn request_approval(
    db: &PgPool,
    agent_did: &str,
    amount: i64,
    currency: &str,
    description: Option<String>,
    webhook_url: Option<String>,
    idempotency_key: Option<String>,
) -> AppResult<serde_json::Value> {
    let approval_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(24);
    let requested_at = Utc::now();

    // Get agent display name
    let agent_name = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT display_name FROM agents WHERE did = $1",
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await?
    .and_then(|(name,)| name)
    .unwrap_or_else(|| agent_did.to_string());

    sqlx::query(
        r#"
        INSERT INTO approvals (id, agent_did, transaction_amount, transaction_currency,
                                transaction_description, status, expires_at, webhook_url, idempotency_key)
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8)
        "#,
    )
    .bind(approval_id)
    .bind(agent_did)
    .bind(amount)
    .bind(currency)
    .bind(&description)
    .bind(expires_at)
    .bind(&webhook_url)
    .bind(&idempotency_key)
    .execute(db)
    .await?;

    // Send webhook notification if URL provided
    if let Some(ref url) = webhook_url {
        let payload = serde_json::json!({
            "type": "approval_requested",
            "approval_id": approval_id.to_string(),
            "agent_did": agent_did,
            "agent_name": agent_name,
            "amount": amount,
            "currency": currency,
            "description": description,
            "approve_url": format!("/approval/{}/approve", approval_id),
            "reject_url": format!("/approval/{}/reject", approval_id),
            "expires_at": expires_at.to_rfc3339(),
        });

        // Fire-and-forget webhook notification
        let url = url.clone();
        let payload_clone = payload.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Err(e) = client.post(&url).json(&payload_clone).send().await {
                tracing::warn!("Approval webhook delivery failed: {}", e);
            }
        });
    }

    Ok(serde_json::json!({
        "approval_id": approval_id.to_string(),
        "status": "pending",
        "expires_at": expires_at.to_rfc3339(),
    }))
}

pub async fn approve(db: &PgPool, approval_id: &str) -> AppResult<serde_json::Value> {
    let id = Uuid::parse_str(approval_id).map_err(|_| AppError::ApprovalNotFound)?;

    let row = sqlx::query_as::<_, (String, chrono::DateTime<Utc>)>(
        "SELECT status, expires_at FROM approvals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::ApprovalNotFound)?;

    let (status, expires_at) = row;
    if status != "pending" {
        return Err(AppError::InvalidRequest(format!(
            "Approval is not pending: {}",
            status
        )));
    }
    if Utc::now() > expires_at {
        return Err(AppError::InvalidRequest("Approval has expired".to_string()));
    }

    let responded_at = Utc::now();
    sqlx::query(
        "UPDATE approvals SET status = 'approved', responded_at = $1 WHERE id = $2",
    )
    .bind(responded_at)
    .bind(id)
    .execute(db)
    .await?;

    Ok(serde_json::json!({
        "approval_id": approval_id,
        "status": "approved",
        "responded_at": responded_at.to_rfc3339(),
    }))
}

pub async fn reject(db: &PgPool, approval_id: &str) -> AppResult<serde_json::Value> {
    let id = Uuid::parse_str(approval_id).map_err(|_| AppError::ApprovalNotFound)?;

    let row = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM approvals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::ApprovalNotFound)?;

    if row.0 != "pending" {
        return Err(AppError::InvalidRequest(format!(
            "Approval is not pending: {}",
            row.0
        )));
    }

    let responded_at = Utc::now();
    sqlx::query(
        "UPDATE approvals SET status = 'rejected', responded_at = $1 WHERE id = $2",
    )
    .bind(responded_at)
    .bind(id)
    .execute(db)
    .await?;

    Ok(serde_json::json!({
        "approval_id": approval_id,
        "status": "rejected",
        "responded_at": responded_at.to_rfc3339(),
    }))
}

/// Background task to expire stale approvals.
pub async fn expire_stale_approvals(db: &PgPool) -> AppResult<u64> {
    let result = sqlx::query(
        "UPDATE approvals SET status = 'expired' WHERE status = 'pending' AND expires_at < NOW()",
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
