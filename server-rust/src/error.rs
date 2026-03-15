use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("DID not found")]
    DIDNotFound,

    #[error("Invalid DID format")]
    InvalidDID,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Scope exceeded: {0}")]
    ScopeExceeded(String),

    #[error("Duplicate transaction")]
    DuplicateTransaction { existing_id: String },

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Chain integrity error")]
    ChainInvalid,

    #[error("Approval required")]
    ApprovalRequired { approval_id: String },

    #[error("Approval pending")]
    ApprovalPending { approval_id: String },

    #[error("Approval not found")]
    ApprovalNotFound,

    #[error("Agent frozen: {0}")]
    AgentFrozen(String),

    #[error("Agent not found")]
    AgentNotFound,

    #[error("Transaction not found")]
    TransactionNotFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::DIDNotFound => (StatusCode::NOT_FOUND, "DID_NOT_FOUND"),
            AppError::InvalidDID => (StatusCode::BAD_REQUEST, "INVALID_DID"),
            AppError::InvalidSignature => (StatusCode::UNAUTHORIZED, "INVALID_SIGNATURE"),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED"),
            AppError::TokenInvalid(_) => (StatusCode::UNAUTHORIZED, "TOKEN_INVALID"),
            AppError::ScopeExceeded(_) => (StatusCode::FORBIDDEN, "SCOPE_EXCEEDED"),
            AppError::DuplicateTransaction { .. } => (StatusCode::CONFLICT, "DUPLICATE_TRANSACTION"),
            AppError::PaymentFailed(_) => (StatusCode::BAD_GATEWAY, "PAYMENT_FAILED"),
            AppError::ChainInvalid => (StatusCode::INTERNAL_SERVER_ERROR, "CHAIN_INVALID"),
            AppError::ApprovalRequired { .. } => (StatusCode::ACCEPTED, "APPROVAL_REQUIRED"),
            AppError::ApprovalPending { .. } => (StatusCode::ACCEPTED, "APPROVAL_PENDING"),
            AppError::ApprovalNotFound => (StatusCode::NOT_FOUND, "APPROVAL_NOT_FOUND"),
            AppError::AgentFrozen(_) => (StatusCode::FORBIDDEN, "AGENT_FROZEN"),
            AppError::AgentNotFound => (StatusCode::NOT_FOUND, "AGENT_NOT_FOUND"),
            AppError::TransactionNotFound => (StatusCode::NOT_FOUND, "TRANSACTION_NOT_FOUND"),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED"),
            AppError::OAuthError(_) => (StatusCode::BAD_REQUEST, "OAUTH_ERROR"),
            AppError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
        };

        let details = match &self {
            AppError::DuplicateTransaction { existing_id } => {
                json!({ "existing_transaction_id": existing_id })
            }
            AppError::ApprovalRequired { approval_id } => {
                json!({ "approval_id": approval_id })
            }
            AppError::ApprovalPending { approval_id } => {
                json!({ "approval_id": approval_id })
            }
            _ => json!({}),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": self.to_string(),
                "details": details
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
