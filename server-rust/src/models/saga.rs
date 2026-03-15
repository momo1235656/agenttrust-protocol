use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Saga {
    pub id: Uuid,
    pub a2a_transfer_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub total_steps: i32,
    pub started_at: DateTime<Utc>,
    pub timeout_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub compensated_at: Option<DateTime<Utc>>,
    pub error_step: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SagaStep {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_number: i32,
    pub step_name: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub compensation_name: Option<String>,
    pub compensation_started_at: Option<DateTime<Utc>>,
    pub compensation_completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
}
