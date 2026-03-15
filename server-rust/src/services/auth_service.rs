use chrono::Utc;
use sqlx::PgPool;

use crate::crypto::jwt::{JwtClaims, JwtKeys, TOKEN_EXPIRY_SECONDS};
use crate::error::{AppError, AppResult};
use crate::services::did_service;

pub const VALID_SCOPES: &[&str] = &[
    "payment:execute",
    "payment:read",
    "balance:read",
    "audit:read",
];

pub async fn issue_token(
    db: &PgPool,
    jwt_keys: &JwtKeys,
    did: &str,
    message: &str,
    signature: &str,
    requested_scopes: Vec<String>,
) -> AppResult<serde_json::Value> {
    // Verify the DID signature
    did_service::verify_did(db, did, message, signature).await?;

    // Get agent from database
    let agent = did_service::get_agent(db, did).await?;

    if !agent.is_active {
        let reason = agent.frozen_reason.unwrap_or_default();
        return Err(AppError::AgentFrozen(reason));
    }

    // Validate requested scopes
    for scope in &requested_scopes {
        if !VALID_SCOPES.contains(&scope.as_str()) {
            return Err(AppError::InvalidRequest(format!(
                "Invalid scope: {}",
                scope
            )));
        }
    }

    let allowed_categories: Vec<String> = agent
        .allowed_categories
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let now = Utc::now().timestamp();
    let claims = JwtClaims {
        sub: did.to_string(),
        scopes: requested_scopes.clone(),
        max_amount: agent.max_transaction_limit,
        currency: "jpy".to_string(),
        allowed_categories,
        iat: now,
        exp: now + TOKEN_EXPIRY_SECONDS,
    };

    let token = jwt_keys.encode(&claims)?;

    Ok(serde_json::json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": TOKEN_EXPIRY_SECONDS,
        "scopes": requested_scopes,
    }))
}

pub fn verify_token(jwt_keys: &JwtKeys, token: &str) -> AppResult<serde_json::Value> {
    let claims = jwt_keys.decode(token)?;
    Ok(serde_json::json!({
        "valid": true,
        "payload": {
            "sub": claims.sub,
            "scopes": claims.scopes,
            "max_amount": claims.max_amount,
            "currency": claims.currency,
            "allowed_categories": claims.allowed_categories,
            "iat": claims.iat,
            "exp": claims.exp,
        }
    }))
}

/// Extract JWT from "Bearer <token>" Authorization header.
pub fn extract_bearer_token(authorization: &str) -> AppResult<&str> {
    authorization
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::TokenInvalid("Missing or invalid Authorization header".to_string()))
}

/// Verify token and return claims.
pub fn decode_claims(jwt_keys: &JwtKeys, authorization: &str) -> AppResult<JwtClaims> {
    let token = extract_bearer_token(authorization)?;
    jwt_keys.decode(token)
}
