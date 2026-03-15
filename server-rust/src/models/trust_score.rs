use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct TrustScore {
    pub agent_did: String,
    pub score: i16,
    pub total_transactions: i64,
    pub successful_transactions: i64,
    pub failed_transactions: i64,
    pub success_rate: f64,
    pub dispute_count: i64,
    pub dispute_rate: f64,
    pub total_volume: i64,
    pub avg_transaction_value: i64,
    pub unique_counterparties: i64,
    pub account_age_days: i32,
    pub calculation_version: String,
    pub calculated_at: DateTime<Utc>,
}
