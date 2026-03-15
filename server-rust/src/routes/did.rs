use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::services::did_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/did/create", post(create_did))
        .route("/did/resolve/*did", get(resolve_did))
        .route("/did/verify", post(verify_did))
}

#[derive(Deserialize)]
struct CreateDIDRequest {
    display_name: Option<String>,
    #[serde(default = "default_max_limit")]
    max_transaction_limit: i64,
    #[serde(default)]
    allowed_categories: Vec<String>,
}

fn default_max_limit() -> i64 {
    100000
}

async fn create_did(
    State(state): State<SharedState>,
    Json(req): Json<CreateDIDRequest>,
) -> AppResult<impl IntoResponse> {
    let result = did_service::create_did(
        &state.db,
        req.display_name,
        req.max_transaction_limit,
        req.allowed_categories,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn resolve_did(
    State(state): State<SharedState>,
    Path(did): Path<String>,
) -> AppResult<impl IntoResponse> {
    let result = did_service::resolve_did(&state.db, &did).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct VerifyDIDRequest {
    did: String,
    message: String,
    signature: String,
}

async fn verify_did(
    State(state): State<SharedState>,
    Json(req): Json<VerifyDIDRequest>,
) -> AppResult<impl IntoResponse> {
    did_service::verify_did(&state.db, &req.did, &req.message, &req.signature).await?;
    Ok(Json(serde_json::json!({
        "did": req.did,
        "verified": true,
    })))
}
