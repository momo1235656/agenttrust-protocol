use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::vc_service;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/vc/issue", post(issue))
        .route("/vc/verify", post(verify))
        .route("/vc/revoke", post(revoke))
}

#[derive(Deserialize)]
struct IssueRequest {
    agent_did: String,
    credential_type: Option<String>,
    expiration_days: Option<i64>,
}

async fn issue(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IssueRequest>,
) -> AppResult<impl IntoResponse> {
    let credential_type = body.credential_type.as_deref().unwrap_or("AgentTrustScore");
    let expiration_days = body.expiration_days.unwrap_or(30);

    // Use server's JWT public key bytes as issuer DID for simplicity
    let issuer_did = format!("did:key:agenttrust-issuer-v1");
    let issuer_private_key = &state.config.jwt_private_key_bytes;

    let vc = vc_service::issue_vc(
        &state.db,
        &body.agent_did,
        &issuer_did,
        issuer_private_key,
        credential_type,
        expiration_days,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "verifiable_credential": vc.credential_json
    }))))
}

#[derive(Deserialize)]
struct VerifyRequest {
    verifiable_credential: serde_json::Value,
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VerifyRequest>,
) -> AppResult<impl IntoResponse> {
    let issuer_public_key = &state.config.jwt_public_key_bytes;
    let result = vc_service::verify_vc(&body.verifiable_credential, issuer_public_key, &state.db).await?;
    Ok((StatusCode::OK, Json(result)))
}

#[derive(Deserialize)]
struct RevokeRequest {
    credential_id: String,
    reason: Option<String>,
}

async fn revoke(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RevokeRequest>,
) -> AppResult<impl IntoResponse> {
    let id = Uuid::parse_str(&body.credential_id)
        .map_err(|_| AppError::BadRequest("Invalid credential_id UUID".to_string()))?;
    let reason = body.reason.as_deref().unwrap_or("Revoked by issuer");
    vc_service::revoke_vc(&state.db, id, reason).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "credential_id": body.credential_id,
        "revoked": true,
        "revoked_at": chrono::Utc::now()
    }))))
}
