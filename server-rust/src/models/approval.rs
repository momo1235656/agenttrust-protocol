use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Approval {
    pub id: Uuid,
    pub agent_did: String,
    pub transaction_amount: i64,
    pub transaction_currency: String,
    pub transaction_description: Option<String>,
    pub status: String,
    pub requested_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub responded_by: Option<String>,
    pub webhook_url: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub idempotency_key: Option<String>,
}
