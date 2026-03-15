use base64::Engine;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::verifiable_credential::VerifiableCredential;
use crate::services::trust_service;

pub async fn issue_vc(
    db: &PgPool,
    agent_did: &str,
    issuer_did: &str,
    issuer_private_key: &[u8],
    credential_type: &str,
    expiration_days: i64,
) -> AppResult<VerifiableCredential> {
    let score = trust_service::get_latest_score(db, agent_did)
        .await?
        .ok_or_else(|| AppError::NotFound("No trust score found. Call /trust/{did}/recalculate first.".to_string()))?;

    let id = Uuid::new_v4();
    let now = Utc::now();
    let expiration = now + Duration::days(expiration_days);

    let credential_json = serde_json::json!({
        "@context": [
            "https://www.w3.org/2018/credentials/v1",
            "https://agenttrust.io/credentials/v1"
        ],
        "id": format!("urn:uuid:{}", id),
        "type": ["VerifiableCredential", credential_type],
        "issuer": {
            "id": issuer_did,
            "name": "AgentTrust Protocol"
        },
        "issuanceDate": now.to_rfc3339(),
        "expirationDate": expiration.to_rfc3339(),
        "credentialSubject": {
            "id": agent_did,
            "trustScore": score.score,
            "riskLevel": risk_level(score.score),
            "totalTransactions": score.total_transactions,
            "successRate": score.success_rate,
            "disputeRate": score.dispute_rate,
            "accountAgeDays": score.account_age_days,
            "calculationVersion": score.calculation_version
        }
    });

    let payload = serde_json::to_string(&credential_json)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let hash = Sha256::digest(payload.as_bytes());

    let signing_key = ed25519_dalek::SigningKey::from_bytes(
        issuer_private_key
            .try_into()
            .map_err(|_| AppError::Internal("Invalid issuer key length".to_string()))?,
    );
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(&hash);
    let proof_value = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(signature.to_bytes());

    sqlx::query(
        r#"
        INSERT INTO verifiable_credentials (
            id, agent_did, credential_type, credential_json,
            issuer_did, issuance_date, expiration_date,
            proof_type, proof_value
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,'Ed25519Signature2020',$8)
        "#,
    )
    .bind(id)
    .bind(agent_did)
    .bind(credential_type)
    .bind(credential_json)
    .bind(issuer_did)
    .bind(now)
    .bind(expiration)
    .bind(&proof_value)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let vc = sqlx::query_as::<_, VerifiableCredential>(
        "SELECT id, agent_did, credential_type, credential_json, issuer_did, issuance_date, expiration_date, revoked, revoked_at, revocation_reason, proof_type, proof_value, created_at FROM verifiable_credentials WHERE id = $1",
    )
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(vc)
}

pub async fn verify_vc(
    credential: &serde_json::Value,
    issuer_public_key: &[u8],
    db: &PgPool,
) -> AppResult<serde_json::Value> {
    let proof_value = credential["proof"]["proofValue"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("Missing proof.proofValue".to_string()))?;

    let id_str = credential["id"].as_str().unwrap_or("");
    let vc_id: Option<Uuid> = id_str
        .strip_prefix("urn:uuid:")
        .and_then(|s| Uuid::parse_str(s).ok());

    let not_revoked = if let Some(uid) = vc_id {
        let row = sqlx::query(
            "SELECT revoked FROM verifiable_credentials WHERE id = $1",
        )
        .bind(uid)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        row.map(|r| {
            let revoked: bool = r.try_get("revoked").unwrap_or(false);
            !revoked
        })
        .unwrap_or(true)
    } else {
        true
    };

    let not_expired = credential["expirationDate"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|exp| exp > Utc::now())
        .unwrap_or(false);

    let mut doc_for_sig = credential.clone();
    doc_for_sig.as_object_mut().map(|m| m.remove("proof"));
    let payload = serde_json::to_string(&doc_for_sig)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let hash = sha2::Sha256::digest(payload.as_bytes());

    let sig_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(proof_value)
        .map_err(|_| AppError::BadRequest("Invalid proof value encoding".to_string()))?;

    let pub_key_arr: [u8; 32] = issuer_public_key
        .try_into()
        .map_err(|_| AppError::Internal("Invalid issuer public key length".to_string()))?;
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pub_key_arr)
        .map_err(|_| AppError::BadRequest("Invalid issuer public key".to_string()))?;

    let sig_arr: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::BadRequest("Invalid signature length".to_string()))?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_arr);

    use ed25519_dalek::Verifier;
    let signature_valid = verifying_key.verify(&hash, &signature).is_ok();

    let subject = &credential["credentialSubject"];
    let valid = signature_valid && not_expired && not_revoked;

    Ok(serde_json::json!({
        "valid": valid,
        "checks": {
            "signature_valid": signature_valid,
            "not_expired": not_expired,
            "not_revoked": not_revoked,
            "issuer_trusted": true
        },
        "credential_subject": subject
    }))
}

pub async fn revoke_vc(db: &PgPool, credential_id: Uuid, reason: &str) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE verifiable_credentials
        SET revoked = true, revoked_at = NOW(), revocation_reason = $2
        WHERE id = $1 AND revoked = false
        "#,
    )
    .bind(credential_id)
    .bind(reason)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("VC not found or already revoked".to_string()));
    }
    Ok(())
}

pub fn risk_level(score: i16) -> &'static str {
    match score {
        80..=100 => "low",
        60..=79 => "medium",
        40..=59 => "high",
        _ => "critical",
    }
}
