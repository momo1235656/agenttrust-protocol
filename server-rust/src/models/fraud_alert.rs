use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct FraudAlert {
    pub id: Uuid,
    pub agent_did: String,
    pub transaction_id: Option<String>,
    pub alert_type: String,
    pub severity: String,
    pub risk_score: f64,
    pub rule_name: String,
    pub details: serde_json::Value,
    pub status: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}
