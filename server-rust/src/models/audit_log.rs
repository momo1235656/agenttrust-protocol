use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub index_num: i64,
    pub agent_did: String,
    pub transaction_id: String,
    pub amount: i64,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub prev_hash: String,
    pub hash: String,
}
