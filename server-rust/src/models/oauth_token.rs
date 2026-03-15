use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct OAuthToken {
    pub id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub client_id: String,
    pub agent_did: String,
    pub scopes: Value,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}
