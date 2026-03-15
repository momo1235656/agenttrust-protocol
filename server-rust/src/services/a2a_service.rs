use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use redis::aio::ConnectionManager;

use crate::error::{AppError, AppResult};
use crate::models::a2a_transfer::A2ATransfer;
use crate::services::{escrow_service, flow_service, kafka_service::KafkaProducer, saga_service, trust_service};

#[derive(Debug, Deserialize)]
pub struct InitiateRequest {
    pub sender_did: String,
    pub receiver_did: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub service_type: Option<String>,
    pub timeout_minutes: Option<i64>,
    pub sender_signature: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InitiateResponse {
    pub transfer_id: Uuid,
    pub saga_id: Uuid,
    pub status: String,
    pub escrow_status: String,
    pub timeout_at: Option<chrono::DateTime<Utc>>,
    pub steps: StepSummary,
}

#[derive(Debug, Serialize)]
pub struct StepSummary {
    pub total: i32,
    pub completed: i32,
    pub current: String,
}

pub async fn initiate(
    db: &PgPool,
    redis: &mut ConnectionManager,
    kafka: &KafkaProducer,
    req: InitiateRequest,
) -> AppResult<InitiateResponse> {
    // Step 1: Flow check
    match flow_service::check(db, redis, &req.sender_did, &req.receiver_did).await? {
        flow_service::FlowDecision::Deny { violations } => {
            return Err(AppError::BadRequest(format!(
                "Flow control violation: {}",
                violations.iter().map(|v| v.rule.as_str()).collect::<Vec<_>>().join(", ")
            )));
        }
        flow_service::FlowDecision::Allow => {}
    }

    // Step 2: Get trust scores for both agents
    let sender_score = trust_service::get_latest_score(db, &req.sender_did)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No trust score for sender {}. Call /trust/{{did}}/recalculate first.", req.sender_did)))?;
    let receiver_score = trust_service::get_latest_score(db, &req.receiver_did)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No trust score for receiver {}. Call /trust/{{did}}/recalculate first.", req.receiver_did)))?;

    // Minimum trust score threshold
    if sender_score.score < 30 {
        return Err(AppError::BadRequest(format!("Sender trust score {} is below minimum threshold 30", sender_score.score)));
    }
    if receiver_score.score < 30 {
        return Err(AppError::BadRequest(format!("Receiver trust score {} is below minimum threshold 30", receiver_score.score)));
    }

    let currency = req.currency.as_deref().unwrap_or("jpy");
    let timeout_minutes = req.timeout_minutes.unwrap_or(60);
    let transfer_id = Uuid::new_v4();

    // Create transfer record
    sqlx::query(
        r#"
        INSERT INTO a2a_transfers (id, sender_did, sender_trust_score, receiver_did, receiver_trust_score,
            amount, currency, description, service_type, status,
            initiated_at, timeout_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'initiated',NOW(),NOW() + ($10 * INTERVAL '1 minute'))
        "#,
    )
    .bind(transfer_id)
    .bind(&req.sender_did)
    .bind(sender_score.score)
    .bind(&req.receiver_did)
    .bind(receiver_score.score)
    .bind(req.amount)
    .bind(currency)
    .bind(&req.description)
    .bind(&req.service_type)
    .bind(timeout_minutes as f64)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Create saga
    let saga = saga_service::create(db, kafka, transfer_id, timeout_minutes).await?;

    // Update transfer with saga_id
    sqlx::query("UPDATE a2a_transfers SET saga_id = $2, updated_at = NOW() WHERE id = $1")
        .bind(transfer_id)
        .bind(saga.id)
        .execute(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Steps 1-4 complete (flow, did verify, trust check, fraud check - synchronous stubs)
    for step in 0..4i32 {
        saga_service::advance_step(db, kafka, saga.id, step).await?;
        saga_service::complete_step(db, kafka, saga.id, step).await?;
    }

    // Step 5: Create escrow
    saga_service::advance_step(db, kafka, saga.id, 4).await?;
    let flow_policy = flow_service::get_policy(db, &req.sender_did).await?;
    let escrow = escrow_service::create(
        db, kafka, transfer_id, req.amount, currency,
        &req.sender_did, &req.receiver_did,
        flow_policy.max_escrow_timeout_hours as i64,
    ).await?;
    saga_service::complete_step(db, kafka, saga.id, 4).await?;

    // Update transfer with escrow_id and escrowed status
    sqlx::query("UPDATE a2a_transfers SET escrow_id = $2, status = 'escrowed', escrowed_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(transfer_id)
        .bind(escrow.id)
        .execute(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Step 6: Notify receiver (stub - in production would send webhook/gRPC)
    saga_service::advance_step(db, kafka, saga.id, 5).await?;
    kafka.send("a2a.transfer", "a2a.transfer.receiver_notified", &transfer_id.to_string(),
        serde_json::json!({ "transfer_id": transfer_id, "receiver_did": req.receiver_did, "amount": req.amount })).await;
    saga_service::complete_step(db, kafka, saga.id, 5).await?;

    // Step 7: await_completion - pause saga
    saga_service::mark_await_completion(db, saga.id).await?;

    let transfer = get_by_id(db, transfer_id).await?;

    kafka.send("a2a.transfer", "a2a.transfer.initiated", &transfer_id.to_string(),
        serde_json::json!({ "transfer_id": transfer_id, "saga_id": saga.id, "amount": req.amount })).await;

    Ok(InitiateResponse {
        transfer_id,
        saga_id: saga.id,
        status: "service_pending".to_string(),
        escrow_status: "funded".to_string(),
        timeout_at: transfer.timeout_at,
        steps: StepSummary {
            total: 10,
            completed: 6,
            current: "await_completion".to_string(),
        },
    })
}

pub async fn get_by_id(db: &PgPool, transfer_id: Uuid) -> AppResult<A2ATransfer> {
    sqlx::query_as::<_, A2ATransfer>("SELECT * FROM a2a_transfers WHERE id = $1")
        .bind(transfer_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Transfer {} not found", transfer_id)))
}

pub async fn complete_transfer(
    db: &PgPool,
    kafka: &KafkaProducer,
    transfer_id: Uuid,
    reporter_did: &str,
    result_summary: &str,
) -> AppResult<A2ATransfer> {
    let transfer = get_by_id(db, transfer_id).await?;

    if transfer.receiver_did != reporter_did {
        return Err(AppError::BadRequest("Only the receiver can complete a transfer".to_string()));
    }
    if transfer.status != "escrowed" && transfer.status != "service_pending" {
        return Err(AppError::BadRequest(format!("Transfer is in status '{}', cannot complete", transfer.status)));
    }

    let saga_id = transfer.saga_id.ok_or_else(|| AppError::Internal("Transfer has no saga".to_string()))?;
    saga_service::complete_by_receiver(db, kafka, saga_id, reporter_did, result_summary).await?;

    // Step 8: Release escrow
    saga_service::advance_step(db, kafka, saga_id, 7).await?;
    if let Some(escrow_id) = transfer.escrow_id {
        escrow_service::release(db, kafka, escrow_id).await?;
    }
    saga_service::complete_step(db, kafka, saga_id, 7).await?;

    // Step 9: Audit record
    saga_service::advance_step(db, kafka, saga_id, 8).await?;
    kafka.send("a2a.transfer", "a2a.transfer.audit_recorded", &transfer_id.to_string(), serde_json::json!({})).await;
    saga_service::complete_step(db, kafka, saga_id, 8).await?;

    // Step 10: Trust update
    saga_service::advance_step(db, kafka, saga_id, 9).await?;
    let _ = trust_service::recalculate_score(db, &transfer.sender_did).await;
    let _ = trust_service::recalculate_score(db, &transfer.receiver_did).await;
    saga_service::complete_step(db, kafka, saga_id, 9).await?;

    // Finalize saga
    saga_service::update_saga_status(db, kafka, saga_id, "completed", None, None).await?;

    // Update transfer status
    sqlx::query(
        "UPDATE a2a_transfers SET status = 'settled', service_completed_at = NOW(), released_at = NOW(), settled_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(transfer_id)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    kafka.send("a2a.transfer", "a2a.transfer.completed", &transfer_id.to_string(),
        serde_json::json!({ "transfer_id": transfer_id, "amount": transfer.amount })).await;

    get_by_id(db, transfer_id).await
}
