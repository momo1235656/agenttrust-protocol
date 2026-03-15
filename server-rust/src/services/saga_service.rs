use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::saga::{Saga, SagaStep};
use crate::services::kafka_service::KafkaProducer;

const STEP_NAMES: &[&str] = &[
    "flow_check",
    "did_verify_both",
    "trust_mutual_check",
    "fraud_check",
    "escrow_fund",
    "notify_receiver",
    "await_completion",
    "escrow_release",
    "audit_record",
    "trust_update",
];

pub async fn create(
    db: &PgPool,
    kafka: &KafkaProducer,
    a2a_transfer_id: Uuid,
    timeout_minutes: i64,
) -> AppResult<Saga> {
    let id = Uuid::new_v4();
    let timeout_at = Utc::now() + Duration::minutes(timeout_minutes);
    let total_steps = STEP_NAMES.len() as i32;

    sqlx::query(
        "INSERT INTO sagas (id, a2a_transfer_id, status, total_steps, timeout_at) VALUES ($1, $2, 'started', $3, $4)"
    )
    .bind(id)
    .bind(a2a_transfer_id)
    .bind(total_steps)
    .bind(timeout_at)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Create all steps as pending
    for (i, name) in STEP_NAMES.iter().enumerate() {
        let compensation = match i {
            4 => Some("escrow_refund"),
            5 => Some("notify_cancel"),
            6 => Some("timeout_refund"),
            _ => None,
        };
        sqlx::query(
            "INSERT INTO saga_steps (id, saga_id, step_number, step_name, status, compensation_name) VALUES ($1, $2, $3, $4, 'pending', $5)"
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(i as i32)
        .bind(name)
        .bind(compensation)
        .execute(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    kafka.send("saga", "saga.started", &id.to_string(), serde_json::json!({ "saga_id": id })).await;
    get_by_id(db, id).await
}

pub async fn get_by_id(db: &PgPool, saga_id: Uuid) -> AppResult<Saga> {
    sqlx::query_as::<_, Saga>("SELECT * FROM sagas WHERE id = $1")
        .bind(saga_id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Saga {} not found", saga_id)))
}

pub async fn get_steps(db: &PgPool, saga_id: Uuid) -> AppResult<Vec<SagaStep>> {
    sqlx::query_as::<_, SagaStep>(
        "SELECT * FROM saga_steps WHERE saga_id = $1 ORDER BY step_number ASC"
    )
    .bind(saga_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_step_status(
    db: &PgPool,
    saga_id: Uuid,
    step_number: i32,
    status: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE saga_steps SET status = $3, started_at = CASE WHEN $3 = 'executing' THEN NOW() ELSE started_at END, completed_at = CASE WHEN $3 IN ('completed','failed','compensated') THEN NOW() ELSE completed_at END WHERE saga_id = $1 AND step_number = $2"
    )
    .bind(saga_id)
    .bind(step_number)
    .bind(status)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn update_saga_status(
    db: &PgPool,
    kafka: &KafkaProducer,
    saga_id: Uuid,
    status: &str,
    error_step: Option<i32>,
    error_message: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE sagas SET
            status = $2,
            current_step = COALESCE($3, current_step),
            error_step = $3,
            error_message = $4,
            completed_at = CASE WHEN $2 = 'completed' THEN NOW() ELSE completed_at END,
            compensated_at = CASE WHEN $2 = 'compensated' THEN NOW() ELSE compensated_at END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(saga_id)
    .bind(status)
    .bind(error_step)
    .bind(error_message)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    kafka.send("saga", &format!("saga.{}", status), &saga_id.to_string(), serde_json::json!({ "saga_id": saga_id })).await;
    Ok(())
}

pub async fn advance_step(
    db: &PgPool,
    kafka: &KafkaProducer,
    saga_id: Uuid,
    step: i32,
) -> AppResult<()> {
    update_step_status(db, saga_id, step, "executing").await?;
    kafka.send("saga", &format!("saga.step.{}.executing", step), &saga_id.to_string(), serde_json::json!({ "step": step })).await;
    Ok(())
}

pub async fn complete_step(
    db: &PgPool,
    kafka: &KafkaProducer,
    saga_id: Uuid,
    step: i32,
) -> AppResult<()> {
    update_step_status(db, saga_id, step, "completed").await?;
    sqlx::query("UPDATE sagas SET current_step = $2, updated_at = NOW() WHERE id = $1")
        .bind(saga_id)
        .bind(step + 1)
        .execute(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    kafka.send("saga", &format!("saga.step.{}.completed", step), &saga_id.to_string(), serde_json::json!({ "step": step })).await;
    Ok(())
}

pub async fn mark_await_completion(db: &PgPool, saga_id: Uuid) -> AppResult<()> {
    // Step 6 (await_completion) is executing, saga paused waiting for receiver
    update_step_status(db, saga_id, 6, "executing").await?;
    sqlx::query("UPDATE sagas SET status = 'in_progress', updated_at = NOW() WHERE id = $1")
        .bind(saga_id)
        .execute(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn complete_by_receiver(
    db: &PgPool,
    kafka: &KafkaProducer,
    saga_id: Uuid,
    reporter_did: &str,
    result_summary: &str,
) -> AppResult<Saga> {
    // Mark await_completion step as completed
    complete_step(db, kafka, saga_id, 6).await?;
    let data = serde_json::json!({ "reporter": reporter_did, "result": result_summary });
    sqlx::query(
        "UPDATE saga_steps SET output_data = $2 WHERE saga_id = $1 AND step_number = 6"
    )
    .bind(saga_id)
    .bind(data)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    kafka.send("saga", "saga.receiver.completed", &saga_id.to_string(), serde_json::json!({ "saga_id": saga_id, "reporter": reporter_did })).await;
    get_by_id(db, saga_id).await
}

pub async fn compensate(
    db: &PgPool,
    kafka: &KafkaProducer,
    saga_id: Uuid,
    failed_step: i32,
    escrow_id: Option<uuid::Uuid>,
    escrow_kafka: &KafkaProducer,
    pool: &PgPool,
) -> AppResult<()> {
    update_saga_status(db, kafka, saga_id, "compensating", Some(failed_step), None).await?;

    // Compensate step 5 (escrow_fund) if it was completed
    let steps = get_steps(db, saga_id).await?;
    for step in steps.iter().rev() {
        if step.step_number >= failed_step {
            continue;
        }
        if step.status == "completed" {
            if let Some(comp) = &step.compensation_name {
                update_step_status(db, saga_id, step.step_number, "compensating").await?;
                match comp.as_str() {
                    "escrow_refund" => {
                        if let Some(eid) = escrow_id {
                            let _ = crate::services::escrow_service::refund(pool, escrow_kafka, eid, "Saga compensation").await;
                        }
                    }
                    "notify_cancel" => {
                        tracing::info!("Saga {}: sending cancel notification (stub)", saga_id);
                    }
                    "timeout_refund" => {
                        if let Some(eid) = escrow_id {
                            let _ = crate::services::escrow_service::refund(pool, escrow_kafka, eid, "Saga timeout").await;
                        }
                    }
                    _ => {}
                }
                update_step_status(db, saga_id, step.step_number, "compensated").await?;
            }
        }
    }

    update_saga_status(db, kafka, saga_id, "compensated", Some(failed_step), None).await?;
    Ok(())
}

pub async fn get_timed_out(db: &PgPool) -> AppResult<Vec<Saga>> {
    sqlx::query_as::<_, Saga>(
        "SELECT * FROM sagas WHERE status IN ('started','in_progress') AND timeout_at < NOW()"
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
