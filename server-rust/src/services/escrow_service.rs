use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::escrow::Escrow;
use crate::services::kafka_service::KafkaProducer;

pub async fn create(
    db: &PgPool,
    kafka: &KafkaProducer,
    a2a_transfer_id: Uuid,
    amount: i64,
    currency: &str,
    payer_did: &str,
    payee_did: &str,
    expires_hours: i64,
) -> AppResult<Escrow> {
    let id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(expires_hours);

    sqlx::query(
        r#"
        INSERT INTO escrows (id, a2a_transfer_id, amount, currency, payer_did, payee_did,
            status, expires_at, funded_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'funded', $7, NOW())
        "#,
    )
    .bind(id)
    .bind(a2a_transfer_id)
    .bind(amount)
    .bind(currency)
    .bind(payer_did)
    .bind(payee_did)
    .bind(expires_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let escrow = get_by_id(db, id).await?;
    kafka.send("escrow", "escrow.funded", &id.to_string(), serde_json::json!({ "escrow_id": id, "amount": amount })).await;
    Ok(escrow)
}

pub async fn release(db: &PgPool, kafka: &KafkaProducer, escrow_id: Uuid) -> AppResult<Escrow> {
    let result = sqlx::query(
        "UPDATE escrows SET status = 'released', released_at = NOW(), updated_at = NOW() WHERE id = $1 AND status = 'funded'"
    )
    .bind(escrow_id)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Escrow not found or not in funded status".to_string()));
    }

    let escrow = get_by_id(db, escrow_id).await?;
    kafka.send("escrow", "escrow.released", &escrow_id.to_string(), serde_json::json!({ "escrow_id": escrow_id })).await;
    Ok(escrow)
}

pub async fn refund(db: &PgPool, kafka: &KafkaProducer, escrow_id: Uuid, reason: &str) -> AppResult<Escrow> {
    let result = sqlx::query(
        "UPDATE escrows SET status = 'refunded', refunded_at = NOW(), dispute_reason = $2, updated_at = NOW() WHERE id = $1 AND status IN ('funded', 'disputed')"
    )
    .bind(escrow_id)
    .bind(reason)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Escrow not found or cannot be refunded".to_string()));
    }

    let escrow = get_by_id(db, escrow_id).await?;
    kafka.send("escrow", "escrow.refunded", &escrow_id.to_string(), serde_json::json!({ "escrow_id": escrow_id, "reason": reason })).await;
    Ok(escrow)
}

pub async fn dispute(
    db: &PgPool,
    kafka: &KafkaProducer,
    escrow_id: Uuid,
    disputed_by: &str,
    reason: &str,
    evidence_hash: Option<&str>,
) -> AppResult<Escrow> {
    let result = sqlx::query(
        "UPDATE escrows SET status = 'disputed', dispute_reason = $2, dispute_opened_at = NOW(), updated_at = NOW() WHERE id = $1 AND status = 'funded'"
    )
    .bind(escrow_id)
    .bind(reason)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Escrow not found or not in funded status".to_string()));
    }

    let escrow = get_by_id(db, escrow_id).await?;
    kafka.send("escrow", "escrow.disputed", &escrow_id.to_string(), serde_json::json!({
        "escrow_id": escrow_id,
        "disputed_by": disputed_by,
        "reason": reason,
        "evidence_hash": evidence_hash
    })).await;
    Ok(escrow)
}

pub async fn get_by_id(db: &PgPool, escrow_id: Uuid) -> AppResult<Escrow> {
    sqlx::query_as::<_, Escrow>("SELECT * FROM escrows WHERE id = $1")
        .bind(escrow_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Escrow {} not found", escrow_id)))
}

pub async fn get_expired_funded(db: &PgPool) -> AppResult<Vec<Escrow>> {
    sqlx::query_as::<_, Escrow>(
        "SELECT * FROM escrows WHERE status = 'funded' AND expires_at < NOW()"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
