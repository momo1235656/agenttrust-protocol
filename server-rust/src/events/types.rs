use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentTrustEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventMetadata {
    pub source: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

impl AgentTrustEvent {
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        payload: serde_json::Value,
        source: impl Into<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            timestamp: chrono::Utc::now(),
            aggregate_id: aggregate_id.into(),
            payload,
            metadata: EventMetadata {
                source: source.into(),
                correlation_id: None,
                causation_id: None,
            },
        }
    }
}
