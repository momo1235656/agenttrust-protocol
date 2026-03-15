use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;

use crate::crypto::jwt::JwtKeys;
use crate::error::{AppError, AppResult};
use crate::services::oauth_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/oauth/authorize", post(authorize))
        .route("/oauth/token", post(token))
        .route("/oauth/revoke", post(revoke))
        .route("/oauth/jwks", get(jwks))
        .route("/oauth/register", post(register_client))
}

#[derive(Deserialize)]
struct AuthorizeParams {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    agent_did: Option<String>,
}

async fn authorize(
    State(state): State<SharedState>,
    Form(params): Form<AuthorizeParams>,
) -> AppResult<impl IntoResponse> {
    if params.response_type != "code" {
        return Err(AppError::OAuthError("unsupported_response_type".to_string()));
    }

    let agent_did = params
        .agent_did
        .ok_or_else(|| AppError::OAuthError("agent_did required".to_string()))?;

    let code = oauth_service::create_authorization_code(
        &state.db,
        &params.client_id,
        &agent_did,
        &params.redirect_uri,
        params.scope.as_deref().unwrap_or("payment:execute"),
        params.state.as_deref(),
        params.code_challenge.as_deref(),
        params.code_challenge_method.as_deref(),
    )
    .await?;

    // Build redirect URL
    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(s) = &params.state {
        redirect_url.push_str(&format!("&state={}", s));
    }

    Ok(Redirect::to(&redirect_url))
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    // Authorization Code Grant
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    // Client Credentials + all grants
    client_id: Option<String>,
    client_secret: Option<String>,
    scope: Option<String>,
    // Client Credentials with DID
    agent_did: Option<String>,
    did_signature: Option<String>,
    // Refresh Token
    refresh_token: Option<String>,
    // Agent Delegation
    delegator_token: Option<String>,
    delegatee_did: Option<String>,
    delegated_scopes: Option<String>,
    delegated_max_amount: Option<i64>,
}

async fn token(
    State(state): State<SharedState>,
    Form(req): Form<TokenRequest>,
) -> AppResult<impl IntoResponse> {
    let client_id = req
        .client_id
        .as_deref()
        .ok_or_else(|| AppError::OAuthError("client_id required".to_string()))?;
    let client_secret = req
        .client_secret
        .as_deref()
        .ok_or_else(|| AppError::OAuthError("client_secret required".to_string()))?;

    let result = match req.grant_type.as_str() {
        "authorization_code" => {
            oauth_service::authorization_code_grant(
                &state.db,
                req.code.as_deref().ok_or_else(|| AppError::OAuthError("code required".to_string()))?,
                req.redirect_uri.as_deref().ok_or_else(|| AppError::OAuthError("redirect_uri required".to_string()))?,
                client_id,
                client_secret,
                req.code_verifier.as_deref(),
            )
            .await?
        }
        "client_credentials" => {
            oauth_service::client_credentials_grant(
                &state.db,
                client_id,
                client_secret,
                req.scope.as_deref().unwrap_or("payment:execute"),
                req.agent_did.as_deref(),
                req.did_signature.as_deref(),
            )
            .await?
        }
        "refresh_token" => {
            oauth_service::refresh_token_grant(
                &state.db,
                req.refresh_token
                    .as_deref()
                    .ok_or_else(|| AppError::OAuthError("refresh_token required".to_string()))?,
                client_id,
                client_secret,
            )
            .await?
        }
        "urn:agenttrust:agent_delegation" => {
            // Agent Delegation: issue a token with reduced scopes on behalf of delegatee
            let delegator_token = req
                .delegator_token
                .as_deref()
                .ok_or_else(|| AppError::OAuthError("delegator_token required".to_string()))?;
            let delegatee_did = req
                .delegatee_did
                .as_deref()
                .ok_or_else(|| AppError::OAuthError("delegatee_did required".to_string()))?;
            let delegated_scopes: Vec<String> = req
                .delegated_scopes
                .as_deref()
                .unwrap_or("payment:execute")
                .split_whitespace()
                .map(String::from)
                .collect();

            // Verify delegator token
            let jwt_keys = JwtKeys::from_bytes(
                &state.config.jwt_private_key_bytes,
                &state.config.jwt_public_key_bytes,
            )?;
            let delegator_claims = jwt_keys.decode(delegator_token)?;

            // Ensure delegated scopes are a subset of delegator's scopes
            for scope in &delegated_scopes {
                if !delegator_claims.scopes.contains(scope) {
                    return Err(AppError::OAuthError(format!(
                        "Cannot delegate scope '{}' that delegator doesn't have",
                        scope
                    )));
                }
            }

            // Issue token for delegatee with reduced scopes
            oauth_service::client_credentials_grant(
                &state.db,
                client_id,
                client_secret,
                &delegated_scopes.join(" "),
                Some(delegatee_did),
                None,
            )
            .await?
        }
        other => {
            return Err(AppError::OAuthError(format!(
                "unsupported_grant_type: {}",
                other
            )))
        }
    };

    Ok(Json(result))
}

#[derive(Deserialize)]
struct RevokeRequest {
    token: String,
    token_type_hint: Option<String>,
    client_id: String,
    client_secret: String,
}

async fn revoke(
    State(state): State<SharedState>,
    Form(req): Form<RevokeRequest>,
) -> AppResult<impl IntoResponse> {
    let result = oauth_service::revoke_token(
        &state.db,
        &req.token,
        req.token_type_hint.as_deref(),
        &req.client_id,
        &req.client_secret,
    )
    .await?;
    Ok(Json(result))
}

async fn jwks(State(state): State<SharedState>) -> impl IntoResponse {
    let jwt_keys = JwtKeys::from_bytes(
        &state.config.jwt_private_key_bytes,
        &state.config.jwt_public_key_bytes,
    )
    .expect("Failed to load JWT keys");

    let jwk = jwt_keys.to_jwk(&state.config.jwt_public_key_bytes);
    Json(serde_json::json!({ "keys": [jwk] }))
}

#[derive(Deserialize)]
struct RegisterClientRequest {
    agent_did: String,
    client_name: String,
    #[serde(default)]
    redirect_uris: Vec<String>,
    #[serde(default = "default_scopes")]
    allowed_scopes: Vec<String>,
}

fn default_scopes() -> Vec<String> {
    vec!["payment:execute".to_string(), "balance:read".to_string()]
}

async fn register_client(
    State(state): State<SharedState>,
    Json(req): Json<RegisterClientRequest>,
) -> AppResult<impl IntoResponse> {
    let result = oauth_service::create_client(
        &state.db,
        &req.agent_did,
        &req.client_name,
        req.redirect_uris,
        req.allowed_scopes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(result)))
}
