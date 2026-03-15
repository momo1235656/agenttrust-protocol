use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct OAuthClient {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret_hash: String,
    pub agent_did: String,
    pub client_name: String,
    pub redirect_uris: Value,
    pub allowed_scopes: Value,
    pub client_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct OAuthAuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub agent_did: String,
    pub redirect_uri: String,
    pub scopes: Value,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}
