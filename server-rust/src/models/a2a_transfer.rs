use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct A2ATransfer {
    pub id: Uuid,
    pub sender_did: String,
    pub sender_trust_score: i16,
    pub receiver_did: String,
    pub receiver_trust_score: i16,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub service_type: Option<String>,
    pub status: String,
    pub escrow_id: Option<Uuid>,
    pub saga_id: Option<Uuid>,
    pub stripe_transfer_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub initiated_at: DateTime<Utc>,
    pub trust_verified_at: Option<DateTime<Utc>>,
    pub escrowed_at: Option<DateTime<Utc>>,
    pub service_completed_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub timeout_at: Option<DateTime<Utc>>,
    pub sender_audit_hash: Option<String>,
    pub receiver_audit_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
