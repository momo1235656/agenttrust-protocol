use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VerifiableCredential {
    pub id: Uuid,
    pub agent_did: String,
    pub credential_type: String,
    pub credential_json: serde_json::Value,
    pub issuer_did: String,
    pub issuance_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    pub proof_type: String,
    pub proof_value: String,
    pub created_at: Option<DateTime<Utc>>,
}
