use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct FlowPolicy {
    pub id: Uuid,
    pub agent_did: Option<String>,
    pub max_transactions_per_minute: i32,
    pub max_transactions_per_hour: i32,
    pub max_transactions_per_day: i32,
    pub max_a2a_with_same_agent_per_day: i32,
    pub max_chain_depth: i32,
    pub max_saga_timeout_minutes: i32,
    pub max_escrow_timeout_hours: i32,
    pub auto_freeze_on_consecutive_failures: i32,
    pub auto_freeze_on_daily_amount_exceed: Option<i64>,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
