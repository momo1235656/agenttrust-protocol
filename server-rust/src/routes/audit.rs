use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::services::audit_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        // /audit/verify must come before /audit/* to prevent "verify" matching as DID
        .route("/audit/verify", post(verify_audit_chain))
        // Use wildcard to capture DIDs with slashes/colons (URL-encoded)
        .route("/audit/*agent_did", get(get_audit_chain))
}

async fn get_audit_chain(
    State(state): State<SharedState>,
    Path(agent_did): Path<String>,
) -> AppResult<impl IntoResponse> {
    // URL-decode the DID (colons are encoded as %3A)
    let decoded = urlencoding_decode(&agent_did);
    let result = audit_service::get_chain(&state.db, &decoded).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct AuditVerifyRequest {
    agent_did: String,
}

async fn verify_audit_chain(
    State(state): State<SharedState>,
    Json(req): Json<AuditVerifyRequest>,
) -> AppResult<impl IntoResponse> {
    let result = audit_service::verify(&state.db, &req.agent_did).await?;
    Ok(Json(result))
}

fn urlencoding_decode(s: &str) -> String {
    // Simple URL decoding for common DID characters
    s.replace("%3A", ":").replace("%2F", "/")
}
