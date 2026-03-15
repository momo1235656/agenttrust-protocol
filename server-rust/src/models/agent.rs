use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub did: String,
    pub public_key: Vec<u8>,
    pub display_name: Option<String>,
    pub did_document: Value,
    pub max_transaction_limit: i64,
    pub daily_transaction_limit: i64,
    pub allowed_categories: Value,
    pub requires_approval_above: i64,
    pub is_active: bool,
    pub frozen_at: Option<DateTime<Utc>>,
    pub frozen_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
