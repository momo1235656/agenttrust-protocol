use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::services::{a2a_service, saga_service};
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/saga/:id/status", get(saga_status))
        .route("/saga/:id/complete", post(complete))
        .route("/saga/:id/compensate", post(compensate_manual))
}

async fn saga_status(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let saga = saga_service::get_by_id(&state.db, id).await?;
    let steps = saga_service::get_steps(&state.db, id).await?;
    Ok(Json(serde_json::json!({
        "id": saga.id,
        "status": saga.status,
        "current_step": saga.current_step,
        "total_steps": saga.total_steps,
        "timeout_at": saga.timeout_at,
        "steps": steps
    })))
}

#[derive(Deserialize)]
struct CompleteRequest {
    reporter_did: String,
    result_summary: Option<String>,
    result_hash: Option<String>,
}

async fn complete(
    State(state): State<SharedState>,
    Path(saga_id): Path<Uuid>,
    Json(req): Json<CompleteRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Find transfer by saga_id
    let transfer = {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT id FROM a2a_transfers WHERE saga_id = $1"
        )
        .bind(saga_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
        .ok_or_else(|| crate::error::AppError::NotFound("Transfer not found for saga".to_string()))?;
        let id: Uuid = row.try_get("id").unwrap();
        id
    };

    let result = a2a_service::complete_transfer(
        &state.db, &state.kafka, transfer,
        &req.reporter_did,
        req.result_summary.as_deref().unwrap_or(""),
    ).await?;

    Ok(Json(serde_json::json!({ "transfer_id": result.id, "status": result.status })))
}

async fn compensate_manual(
    State(state): State<SharedState>,
    Path(saga_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let saga = saga_service::get_by_id(&state.db, saga_id).await?;
    let transfer = sqlx::query_as::<_, crate::models::a2a_transfer::A2ATransfer>(
        "SELECT * FROM a2a_transfers WHERE saga_id = $1"
    )
    .bind(saga_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
    .ok_or_else(|| crate::error::AppError::NotFound("Transfer not found".to_string()))?;

    saga_service::compensate(
        &state.db, &state.kafka, saga_id,
        saga.current_step,
        transfer.escrow_id,
        &state.kafka,
        &state.db,
    ).await?;

    Ok(Json(serde_json::json!({ "saga_id": saga_id, "status": "compensating" })))
}
