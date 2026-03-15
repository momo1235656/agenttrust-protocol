use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::escrow::Escrow;
use crate::services::escrow_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/escrow/:id/release", post(release))
        .route("/escrow/:id/refund", post(refund))
        .route("/escrow/:id/dispute", post(dispute))
}

async fn release(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Escrow>> {
    let escrow = escrow_service::release(&state.db, &state.kafka, id).await?;
    Ok(Json(escrow))
}

#[derive(Deserialize)]
struct RefundRequest {
    reason: Option<String>,
}

async fn refund(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RefundRequest>,
) -> AppResult<Json<Escrow>> {
    let escrow = escrow_service::refund(&state.db, &state.kafka, id, req.reason.as_deref().unwrap_or("Manual refund")).await?;
    Ok(Json(escrow))
}

#[derive(Deserialize)]
struct DisputeRequest {
    disputed_by: String,
    reason: String,
    evidence_hash: Option<String>,
}

async fn dispute(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DisputeRequest>,
) -> AppResult<Json<Escrow>> {
    let escrow = escrow_service::dispute(&state.db, &state.kafka, id, &req.disputed_by, &req.reason, req.evidence_hash.as_deref()).await?;
    Ok(Json(escrow))
}
