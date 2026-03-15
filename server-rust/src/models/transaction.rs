use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub agent_did: String,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub status: String,
    pub payment_provider: Option<String>,
    pub provider_payment_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub approval_id: Option<Uuid>,
    pub audit_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
