use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::services::approval_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/approval/request", post(request_approval))
        .route("/approval/:id/approve", post(approve))
        .route("/approval/:id/reject", post(reject))
}

#[derive(Deserialize)]
struct ApprovalRequest {
    agent_did: String,
    amount: i64,
    #[serde(default = "default_currency")]
    currency: String,
    description: Option<String>,
    webhook_url: Option<String>,
    idempotency_key: Option<String>,
}

fn default_currency() -> String {
    "jpy".to_string()
}

async fn request_approval(
    State(state): State<SharedState>,
    Json(req): Json<ApprovalRequest>,
) -> AppResult<impl IntoResponse> {
    let result = approval_service::request_approval(
        &state.db,
        &req.agent_did,
        req.amount,
        &req.currency,
        req.description,
        req.webhook_url,
        req.idempotency_key,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn approve(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let result = approval_service::approve(&state.db, &id).await?;
    Ok(Json(result))
}

async fn reject(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let result = approval_service::reject(&state.db, &id).await?;
    Ok(Json(result))
}
