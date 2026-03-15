use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::crypto::jwt::JwtKeys;
use crate::error::{AppError, AppResult};
use crate::payment_providers::stripe_provider::StripeProvider;
use crate::services::payment_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/payment/execute", post(execute_payment))
        .route("/payment/refund", post(refund_payment))
        .route("/payment/methods", get(payment_methods))
        // Must come after /payment/refund and /payment/methods to avoid capture conflict
        .route("/payment/:transaction_id", get(get_payment_status))
}

#[derive(Deserialize)]
struct PaymentExecuteRequest {
    amount: i64,
    #[serde(default = "default_currency")]
    currency: String,
    #[serde(default)]
    description: String,
    #[serde(default = "new_idempotency_key")]
    idempotency_key: String,
}

fn default_currency() -> String {
    "jpy".to_string()
}

fn new_idempotency_key() -> String {
    Uuid::new_v4().to_string()
}

async fn execute_payment(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<PaymentExecuteRequest>,
) -> AppResult<impl IntoResponse> {
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::TokenInvalid("Missing Authorization header".to_string()))?;

    let jwt_keys = JwtKeys::from_bytes(
        &state.config.jwt_private_key_bytes,
        &state.config.jwt_public_key_bytes,
    )?;

    let provider = StripeProvider::new(state.config.stripe_secret_key());

    // Check circuit breaker before calling Stripe
    if !state.stripe_circuit_breaker.is_allowed() {
        return Err(AppError::PaymentFailed(
            "Payment service temporarily unavailable (circuit open)".to_string(),
        ));
    }

    let result = payment_service::execute_payment(
        &state.db,
        &jwt_keys,
        &provider,
        state.config.approval_required_above,
        authorization,
        req.amount,
        &req.currency,
        &req.description,
        &req.idempotency_key,
    )
    .await;

    match &result {
        Ok(_) => state.stripe_circuit_breaker.record_success(),
        Err(AppError::PaymentFailed(_)) => state.stripe_circuit_breaker.record_failure(),
        _ => {}
    }

    Ok(Json(result?))
}

async fn get_payment_status(
    State(state): State<SharedState>,
    Path(transaction_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let result = payment_service::get_payment_status(&state.db, &transaction_id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct RefundRequest {
    transaction_id: String,
    amount: Option<i64>,
    reason: Option<String>,
}

async fn refund_payment(
    State(state): State<SharedState>,
    Json(req): Json<RefundRequest>,
) -> AppResult<impl IntoResponse> {
    let provider = StripeProvider::new(state.config.stripe_secret_key());
    let result = payment_service::refund_payment(
        &state.db,
        &provider,
        &req.transaction_id,
        req.amount,
        req.reason.as_deref().unwrap_or("requested_by_customer"),
    )
    .await?;
    Ok(Json(result))
}

async fn payment_methods() -> impl IntoResponse {
    Json(serde_json::json!({
        "methods": [
            { "id": "stripe", "name": "クレジットカード", "currencies": ["jpy", "usd"] },
            { "id": "paypay", "name": "PayPay", "currencies": ["jpy"] }
        ]
    }))
}
