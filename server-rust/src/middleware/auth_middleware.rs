use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::error::AppError;
use crate::state::SharedState;

/// Middleware extension that carries decoded JWT claims.
#[derive(Clone, Debug)]
pub struct AuthenticatedAgent {
    pub did: String,
    pub scopes: Vec<String>,
    pub max_amount: i64,
}

/// JWT authentication middleware.
/// Extracts and validates the Bearer token, attaches claims to request extensions.
pub async fn auth_middleware(
    State(state): State<SharedState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let authorization = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::TokenInvalid("Missing Authorization header".to_string()))?;

    let jwt_keys = crate::crypto::jwt::JwtKeys::from_bytes(
        &state.config.jwt_private_key_bytes,
        &state.config.jwt_public_key_bytes,
    )?;

    let claims = crate::services::auth_service::decode_claims(&jwt_keys, authorization)?;

    request.extensions_mut().insert(AuthenticatedAgent {
        did: claims.sub,
        scopes: claims.scopes,
        max_amount: claims.max_amount,
    });

    Ok(next.run(request).await)
}
