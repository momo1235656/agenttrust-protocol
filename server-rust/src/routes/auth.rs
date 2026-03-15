use axum::{
    extract::State,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::crypto::jwt::JwtKeys;
use crate::error::AppResult;
use crate::services::auth_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/auth/token", post(issue_token))
        .route("/auth/verify-token", post(verify_token))
}

#[derive(Deserialize)]
struct TokenRequest {
    did: String,
    message: String,
    signature: String,
    #[serde(default = "default_scopes")]
    requested_scopes: Vec<String>,
}

fn default_scopes() -> Vec<String> {
    vec!["payment:execute".to_string()]
}

async fn issue_token(
    State(state): State<SharedState>,
    Json(req): Json<TokenRequest>,
) -> AppResult<impl IntoResponse> {
    let jwt_keys = JwtKeys::from_bytes(
        &state.config.jwt_private_key_bytes,
        &state.config.jwt_public_key_bytes,
    )?;

    let result = auth_service::issue_token(
        &state.db,
        &jwt_keys,
        &req.did,
        &req.message,
        &req.signature,
        req.requested_scopes,
    )
    .await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
struct VerifyTokenRequest {
    token: String,
}

async fn verify_token(
    State(state): State<SharedState>,
    Json(req): Json<VerifyTokenRequest>,
) -> AppResult<impl IntoResponse> {
    let jwt_keys = JwtKeys::from_bytes(
        &state.config.jwt_private_key_bytes,
        &state.config.jwt_public_key_bytes,
    )?;

    let result = auth_service::verify_token(&jwt_keys, &req.token)?;
    Ok(Json(result))
}
