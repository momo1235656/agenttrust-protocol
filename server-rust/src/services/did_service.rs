use serde_json::json;
use sqlx::PgPool;

use crate::crypto::keys::{verifying_key_from_base64, AgentKeyPair};
use crate::crypto::signing::verify_signature_b64;
use crate::error::{AppError, AppResult};
use crate::models::agent::Agent;

pub async fn create_did(
    db: &PgPool,
    display_name: Option<String>,
    max_transaction_limit: i64,
    allowed_categories: Vec<String>,
) -> AppResult<serde_json::Value> {
    let keypair = AgentKeyPair::generate();
    let did = keypair.did.clone();
    let public_key_b64 = keypair.public_key_base64();
    let private_key_b64 = keypair.private_key_base64();
    let public_key_bytes = keypair.public_key_bytes().to_vec();

    let document = json!({
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": did,
        "authentication": [{
            "type": "Ed25519VerificationKey2020",
            "publicKeyBase64": public_key_b64,
        }],
        "service": [{
            "type": "AgentPayment",
            "maxTransactionLimit": max_transaction_limit,
            "allowedCategories": allowed_categories,
        }]
    });

    let allowed_cats_json = serde_json::to_value(&allowed_categories)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO agents (did, public_key, display_name, did_document, max_transaction_limit,
                            daily_transaction_limit, allowed_categories, requires_approval_above,
                            is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)
        "#,
    )
    .bind(&did)
    .bind(&public_key_bytes)
    .bind(&display_name)
    .bind(&document)
    .bind(max_transaction_limit)
    .bind(max_transaction_limit * 5) // daily limit = 5x single limit
    .bind(&allowed_cats_json)
    .bind(30000_i64) // default approval threshold
    .execute(db)
    .await?;

    Ok(json!({
        "did": did,
        "document": document,
        "private_key_base64": private_key_b64,
    }))
}

pub async fn resolve_did(db: &PgPool, did: &str) -> AppResult<serde_json::Value> {
    let row = sqlx::query_as::<_, (serde_json::Value,)>(
        "SELECT did_document FROM agents WHERE did = $1",
    )
    .bind(did)
    .fetch_optional(db)
    .await?;

    match row {
        Some((document,)) => Ok(json!({
            "did": did,
            "document": document,
            "found": true,
        })),
        None => Err(AppError::DIDNotFound),
    }
}

pub async fn verify_did(
    db: &PgPool,
    did: &str,
    message: &str,
    signature: &str,
) -> AppResult<()> {
    let row = sqlx::query_as::<_, (serde_json::Value,)>(
        "SELECT did_document FROM agents WHERE did = $1",
    )
    .bind(did)
    .fetch_optional(db)
    .await?;

    let (document,) = row.ok_or(AppError::DIDNotFound)?;

    let auth = document
        .get("authentication")
        .and_then(|a| a.as_array())
        .ok_or(AppError::InvalidSignature)?;

    let public_key_b64 = auth
        .first()
        .and_then(|a| a.get("publicKeyBase64"))
        .and_then(|v| v.as_str())
        .ok_or(AppError::InvalidSignature)?;

    let verifying_key = verifying_key_from_base64(public_key_b64)?;
    verify_signature_b64(&verifying_key, message, signature)
}

pub async fn get_agent(db: &PgPool, did: &str) -> AppResult<Agent> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE did = $1")
        .bind(did)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::AgentNotFound)
}

pub async fn get_verifying_key_from_did_doc(document: &serde_json::Value) -> AppResult<ed25519_dalek::VerifyingKey> {
    let auth = document
        .get("authentication")
        .and_then(|a| a.as_array())
        .ok_or(AppError::InvalidSignature)?;

    let public_key_b64 = auth
        .first()
        .and_then(|a| a.get("publicKeyBase64"))
        .and_then(|v| v.as_str())
        .ok_or(AppError::InvalidSignature)?;

    verifying_key_from_base64(public_key_b64)
}
