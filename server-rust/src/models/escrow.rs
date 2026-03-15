use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Escrow {
    pub id: Uuid,
    pub a2a_transfer_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub payer_did: String,
    pub payee_did: String,
    pub status: String,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub funded_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub dispute_reason: Option<String>,
    pub dispute_opened_at: Option<DateTime<Utc>>,
    pub dispute_resolved_at: Option<DateTime<Utc>>,
    pub dispute_resolution: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
