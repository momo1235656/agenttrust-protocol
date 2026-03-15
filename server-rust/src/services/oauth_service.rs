use base64::Engine;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

const ACCESS_TOKEN_EXPIRY_SECS: i64 = 1800; // 30 min
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_opaque_token(prefix: &str) -> String {
    use rand::RngCore;
    let mut random_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut random_bytes);
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    format!("{}_{}", prefix, b64.encode(random_bytes))
}

/// Verify client credentials (client_id + client_secret).
async fn verify_client(db: &PgPool, client_id: &str, client_secret: &str) -> AppResult<crate::models::oauth_client::OAuthClient> {
    use crate::models::oauth_client::OAuthClient;
    let client = sqlx::query_as::<_, OAuthClient>(
        "SELECT * FROM oauth_clients WHERE client_id = $1 AND is_active = true",
    )
    .bind(client_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::OAuthError("invalid_client".to_string()))?;

    let valid = bcrypt::verify(client_secret, &client.client_secret_hash)
        .map_err(|_| AppError::OAuthError("invalid_client".to_string()))?;

    if !valid {
        return Err(AppError::OAuthError("invalid_client".to_string()));
    }

    Ok(client)
}

/// Issue tokens (access + refresh) and store them.
async fn issue_tokens(
    db: &PgPool,
    client_id: &str,
    agent_did: &str,
    scopes: Vec<String>,
) -> AppResult<serde_json::Value> {
    let access_token = generate_opaque_token("at");
    let refresh_token = generate_opaque_token("rt");
    let access_hash = hash_token(&access_token);
    let refresh_hash = hash_token(&refresh_token);

    let now = Utc::now();
    let access_expires = now + Duration::seconds(ACCESS_TOKEN_EXPIRY_SECS);
    let refresh_expires = now + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);

    let scopes_json = serde_json::to_value(&scopes)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO oauth_tokens (access_token_hash, refresh_token_hash, client_id, agent_did,
                                   scopes, access_token_expires_at, refresh_token_expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&access_hash)
    .bind(&refresh_hash)
    .bind(client_id)
    .bind(agent_did)
    .bind(&scopes_json)
    .bind(access_expires)
    .bind(refresh_expires)
    .execute(db)
    .await?;

    Ok(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": ACCESS_TOKEN_EXPIRY_SECS,
        "refresh_token": refresh_token,
        "scope": scopes.join(" "),
    }))
}

/// Client Credentials Grant (primary flow for agents).
pub async fn client_credentials_grant(
    db: &PgPool,
    client_id: &str,
    client_secret: &str,
    scope: &str,
    agent_did: Option<&str>,
    did_signature: Option<&str>,
) -> AppResult<serde_json::Value> {
    let client = verify_client(db, client_id, client_secret).await?;

    let allowed_scopes: Vec<String> = client
        .allowed_scopes
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let requested_scopes: Vec<String> = scope.split_whitespace().map(String::from).collect();
    for s in &requested_scopes {
        if !allowed_scopes.contains(s) {
            return Err(AppError::OAuthError(format!("invalid_scope: {}", s)));
        }
    }

    // Use client's associated agent DID if not specified
    let did = agent_did.unwrap_or(&client.agent_did);

    // Optionally verify DID signature if provided
    if let Some(signature) = did_signature {
        // Verify the signature against the client_id as the message
        // (using client_id as the challenge message)
        // Note: In production this would verify against a proper challenge
        crate::services::did_service::verify_did(db, did, client_id, signature).await?;
    }

    issue_tokens(db, client_id, did, requested_scopes).await
}

/// Authorization Code Grant - step 1: create authorization code.
pub async fn create_authorization_code(
    db: &PgPool,
    client_id: &str,
    agent_did: &str,
    redirect_uri: &str,
    scope: &str,
    state: Option<&str>,
    code_challenge: Option<&str>,
    code_challenge_method: Option<&str>,
) -> AppResult<String> {
    // Verify client exists
    let row = sqlx::query_as::<_, (String, serde_json::Value, serde_json::Value)>(
        "SELECT client_id, redirect_uris, allowed_scopes FROM oauth_clients WHERE client_id = $1 AND is_active = true",
    )
    .bind(client_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::OAuthError("invalid_client".to_string()))?;

    let (_client_id_check, redirect_uris_json, allowed_scopes_json) = row;

    // Validate redirect_uri
    let redirect_uris: Vec<String> = redirect_uris_json
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    if !redirect_uris.is_empty() && !redirect_uris.contains(&redirect_uri.to_string()) {
        return Err(AppError::OAuthError("invalid_redirect_uri".to_string()));
    }

    let allowed_scopes: Vec<String> = allowed_scopes_json
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let requested_scopes: Vec<String> = scope.split_whitespace().map(String::from).collect();
    for s in &requested_scopes {
        if !allowed_scopes.contains(s) {
            return Err(AppError::OAuthError(format!("invalid_scope: {}", s)));
        }
    }

    let code = generate_opaque_token("code");
    let expires_at = Utc::now() + Duration::minutes(10);
    let scopes_json = serde_json::to_value(&requested_scopes)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO oauth_authorization_codes
            (code, client_id, agent_did, redirect_uri, scopes, code_challenge,
             code_challenge_method, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&code)
    .bind(client_id)
    .bind(agent_did)
    .bind(redirect_uri)
    .bind(&scopes_json)
    .bind(code_challenge)
    .bind(code_challenge_method.unwrap_or("S256"))
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(code)
}

/// Authorization Code Grant - step 2: exchange code for tokens.
pub async fn authorization_code_grant(
    db: &PgPool,
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
    code_verifier: Option<&str>,
) -> AppResult<serde_json::Value> {
    verify_client(db, client_id, client_secret).await?;

    use crate::models::oauth_client::OAuthAuthorizationCode;
    let auth_code = sqlx::query_as::<_, OAuthAuthorizationCode>(
        "SELECT * FROM oauth_authorization_codes WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::OAuthError("invalid_grant".to_string()))?;

    if auth_code.used {
        return Err(AppError::OAuthError("invalid_grant: code already used".to_string()));
    }
    if auth_code.client_id != client_id {
        return Err(AppError::OAuthError("invalid_grant".to_string()));
    }
    if auth_code.redirect_uri != redirect_uri {
        return Err(AppError::OAuthError("invalid_redirect_uri".to_string()));
    }
    if Utc::now() > auth_code.expires_at {
        return Err(AppError::OAuthError("invalid_grant: code expired".to_string()));
    }

    // PKCE verification
    if let Some(challenge) = &auth_code.code_challenge {
        let verifier = code_verifier
            .ok_or_else(|| AppError::OAuthError("invalid_grant: code_verifier required".to_string()))?;

        // S256: challenge = BASE64URL(SHA256(verifier))
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let digest = hasher.finalize();
        let b64url = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let computed = b64url.encode(digest);

        if computed != *challenge {
            return Err(AppError::OAuthError("invalid_grant: code_verifier mismatch".to_string()));
        }
    }

    // Mark code as used
    sqlx::query("UPDATE oauth_authorization_codes SET used = true WHERE code = $1")
        .bind(code)
        .execute(db)
        .await?;

    let scopes: Vec<String> = auth_code
        .scopes
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    issue_tokens(db, client_id, &auth_code.agent_did, scopes).await
}

/// Refresh Token Grant.
pub async fn refresh_token_grant(
    db: &PgPool,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> AppResult<serde_json::Value> {
    verify_client(db, client_id, client_secret).await?;

    let token_hash = hash_token(refresh_token);
    let token_row = sqlx::query_as::<_, (String, chrono::DateTime<Utc>, bool, serde_json::Value)>(
        r#"
        SELECT agent_did, refresh_token_expires_at, revoked, scopes
        FROM oauth_tokens
        WHERE refresh_token_hash = $1 AND client_id = $2
        "#,
    )
    .bind(&token_hash)
    .bind(client_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::OAuthError("invalid_grant".to_string()))?;

    let (agent_did, refresh_expires, revoked, scopes_json) = token_row;

    if revoked {
        return Err(AppError::OAuthError("invalid_grant: token revoked".to_string()));
    }
    if Utc::now() > refresh_expires {
        return Err(AppError::OAuthError("invalid_grant: refresh token expired".to_string()));
    }

    // Revoke old token
    sqlx::query("UPDATE oauth_tokens SET revoked = true WHERE refresh_token_hash = $1")
        .bind(&token_hash)
        .execute(db)
        .await?;

    let scopes: Vec<String> = scopes_json
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    issue_tokens(db, client_id, &agent_did, scopes).await
}

/// Revoke a token (access or refresh).
pub async fn revoke_token(
    db: &PgPool,
    token: &str,
    token_type_hint: Option<&str>,
    client_id: &str,
    client_secret: &str,
) -> AppResult<serde_json::Value> {
    verify_client(db, client_id, client_secret).await?;

    let token_hash = hash_token(token);

    // Try to revoke as access token first (or based on hint)
    let access_rows = sqlx::query("UPDATE oauth_tokens SET revoked = true WHERE access_token_hash = $1 AND client_id = $2")
        .bind(&token_hash)
        .bind(client_id)
        .execute(db)
        .await?
        .rows_affected();

    if access_rows == 0 {
        // Try refresh token
        sqlx::query("UPDATE oauth_tokens SET revoked = true WHERE refresh_token_hash = $1 AND client_id = $2")
            .bind(&token_hash)
            .bind(client_id)
            .execute(db)
            .await?;
    }

    Ok(serde_json::json!({ "revoked": true }))
}

/// Create a new OAuth client (for registering agents).
pub async fn create_client(
    db: &PgPool,
    agent_did: &str,
    client_name: &str,
    redirect_uris: Vec<String>,
    allowed_scopes: Vec<String>,
) -> AppResult<serde_json::Value> {
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let client_id = format!("client_{}", b64.encode(Uuid::new_v4().as_bytes()));
    let raw_secret: [u8; 32] = {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        bytes
    };
    let client_secret = format!("secret_{}", b64.encode(raw_secret));
    let secret_hash = bcrypt::hash(&client_secret, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let uris_json = serde_json::to_value(&redirect_uris)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let scopes_json = serde_json::to_value(&allowed_scopes)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO oauth_clients (client_id, client_secret_hash, agent_did, client_name,
                                    redirect_uris, allowed_scopes)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&client_id)
    .bind(&secret_hash)
    .bind(agent_did)
    .bind(client_name)
    .bind(&uris_json)
    .bind(&scopes_json)
    .execute(db)
    .await?;

    Ok(serde_json::json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "client_name": client_name,
        "agent_did": agent_did,
    }))
}
