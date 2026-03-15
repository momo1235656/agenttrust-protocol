use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::jwt::JwtKeys;
use crate::error::{AppError, AppResult};
use crate::payment_providers::traits::PaymentProvider;
use crate::services::{approval_service, audit_service, auth_service};

pub async fn execute_payment(
    db: &PgPool,
    jwt_keys: &JwtKeys,
    payment_provider: &dyn PaymentProvider,
    approval_threshold: i64,
    authorization: &str,
    amount: i64,
    currency: &str,
    description: &str,
    idempotency_key: &str,
) -> AppResult<serde_json::Value> {
    // Verify JWT
    let claims = auth_service::decode_claims(jwt_keys, authorization)?;
    let agent_did = &claims.sub;

    // Check scope
    if !claims.scopes.contains(&"payment:execute".to_string()) {
        return Err(AppError::ScopeExceeded(
            "Token does not have payment:execute scope".to_string(),
        ));
    }

    // Check amount limit
    if amount > claims.max_amount {
        return Err(AppError::ScopeExceeded(format!(
            "Amount {} exceeds token max_amount {}",
            amount, claims.max_amount
        )));
    }

    // Check agent is active
    let agent = sqlx::query_as::<_, (bool, Option<String>)>(
        "SELECT is_active, frozen_reason FROM agents WHERE did = $1",
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::AgentNotFound)?;

    if !agent.0 {
        return Err(AppError::AgentFrozen(agent.1.unwrap_or_default()));
    }

    // Check for duplicate idempotency key
    let existing = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM transactions WHERE idempotency_key = $1",
    )
    .bind(idempotency_key)
    .fetch_optional(db)
    .await?;

    if let Some((existing_id,)) = existing {
        return Err(AppError::DuplicateTransaction {
            existing_id,
        });
    }

    // Check if approval is required
    if amount >= approval_threshold {
        let approval = approval_service::request_approval(
            db,
            agent_did,
            amount,
            currency,
            Some(description.to_string()),
            None, // webhook_url - could be from agent config
            Some(idempotency_key.to_string()),
        )
        .await?;
        return Err(AppError::ApprovalRequired {
            approval_id: approval["approval_id"].as_str().unwrap_or("").to_string(),
        });
    }

    // Execute payment
    let transaction_id = format!("tx_{}", Uuid::new_v4().to_string().replace('-', "").chars().take(20).collect::<String>());
    let payment_result = payment_provider
        .execute(amount, currency, description, idempotency_key)
        .await?;

    let status = if ["succeeded", "requires_capture"].contains(&payment_result.status.as_str()) {
        "succeeded"
    } else {
        "failed"
    };

    let created_at = Utc::now();

    // Insert transaction
    sqlx::query(
        r#"
        INSERT INTO transactions (id, agent_did, amount, currency, description, status,
                                   payment_provider, provider_payment_id, idempotency_key)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(&transaction_id)
    .bind(agent_did)
    .bind(amount)
    .bind(currency)
    .bind(description)
    .bind(status)
    .bind(payment_provider.name())
    .bind(&payment_result.provider_payment_id)
    .bind(idempotency_key)
    .execute(db)
    .await?;

    // Record in audit hash chain
    let audit_hash = audit_service::record(
        db,
        agent_did,
        &transaction_id,
        amount,
        status,
        created_at,
    )
    .await?;

    // Update transaction with audit hash
    sqlx::query("UPDATE transactions SET audit_hash = $1 WHERE id = $2")
        .bind(&audit_hash)
        .bind(&transaction_id)
        .execute(db)
        .await?;

    Ok(serde_json::json!({
        "transaction_id": transaction_id,
        "status": status,
        "amount": amount,
        "currency": currency,
        "agent_did": agent_did,
        "stripe_payment_intent_id": payment_result.provider_payment_id,
        "audit_hash": audit_hash,
        "created_at": created_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    }))
}

pub async fn get_payment_status(db: &PgPool, transaction_id: &str) -> AppResult<serde_json::Value> {
    let row = sqlx::query_as::<_, (String, i64, String, String, chrono::DateTime<Utc>)>(
        "SELECT status, amount, currency, agent_did, created_at FROM transactions WHERE id = $1",
    )
    .bind(transaction_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::TransactionNotFound)?;

    let (status, amount, currency, agent_did, created_at) = row;
    Ok(serde_json::json!({
        "transaction_id": transaction_id,
        "status": status,
        "amount": amount,
        "currency": currency,
        "agent_did": agent_did,
        "created_at": created_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    }))
}

pub async fn refund_payment(
    db: &PgPool,
    payment_provider: &dyn PaymentProvider,
    transaction_id: &str,
    amount: Option<i64>,
    reason: &str,
) -> AppResult<serde_json::Value> {
    let row = sqlx::query_as::<_, (String, Option<String>, i64)>(
        "SELECT status, provider_payment_id, amount FROM transactions WHERE id = $1",
    )
    .bind(transaction_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::TransactionNotFound)?;

    let (status, provider_payment_id, original_amount) = row;
    if status != "succeeded" {
        return Err(AppError::PaymentFailed(format!(
            "Cannot refund transaction with status: {}",
            status
        )));
    }

    let provider_id = provider_payment_id.ok_or_else(|| {
        AppError::PaymentFailed("No provider payment ID for this transaction".to_string())
    })?;

    let refund_result = payment_provider
        .refund(&provider_id, amount, reason)
        .await?;

    // Update transaction status
    sqlx::query("UPDATE transactions SET status = 'refunded', updated_at = NOW() WHERE id = $1")
        .bind(transaction_id)
        .execute(db)
        .await?;

    Ok(serde_json::json!({
        "transaction_id": transaction_id,
        "refund_id": refund_result.provider_refund_id,
        "status": "refunded",
        "amount": refund_result.amount,
    }))
}
