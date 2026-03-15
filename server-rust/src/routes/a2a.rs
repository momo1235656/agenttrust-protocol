use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::services::a2a_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/a2a/transfer", post(initiate_transfer))
        .route("/a2a/transfer/:id", get(get_transfer))
}

async fn initiate_transfer(
    State(state): State<SharedState>,
    Json(req): Json<a2a_service::InitiateRequest>,
) -> AppResult<(StatusCode, Json<a2a_service::InitiateResponse>)> {
    let mut redis = state.redis.clone();
    let result = a2a_service::initiate(&state.db, &mut redis, &state.kafka, req).await?;
    Ok((StatusCode::ACCEPTED, Json(result)))
}

async fn get_transfer(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let transfer = a2a_service::get_by_id(&state.db, id).await?;

    // Also fetch saga if present
    let saga_info = if let Some(saga_id) = transfer.saga_id {
        match crate::services::saga_service::get_by_id(&state.db, saga_id).await {
            Ok(saga) => {
                let steps = crate::services::saga_service::get_steps(&state.db, saga_id).await.unwrap_or_default();
                serde_json::json!({
                    "id": saga.id,
                    "status": saga.status,
                    "current_step": saga.current_step,
                    "total_steps": saga.total_steps,
                    "steps": steps
                })
            }
            Err(_) => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    };

    let escrow_info = if let Some(escrow_id) = transfer.escrow_id {
        match crate::services::escrow_service::get_by_id(&state.db, escrow_id).await {
            Ok(escrow) => serde_json::json!({
                "id": escrow.id,
                "status": escrow.status,
                "funded_at": escrow.funded_at
            }),
            Err(_) => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    };

    Ok(Json(serde_json::json!({
        "transfer_id": transfer.id,
        "sender_did": transfer.sender_did,
        "receiver_did": transfer.receiver_did,
        "amount": transfer.amount,
        "currency": transfer.currency,
        "status": transfer.status,
        "escrow": escrow_info,
        "saga": saga_info,
        "timeout_at": transfer.timeout_at,
        "initiated_at": transfer.initiated_at
    })))
}
