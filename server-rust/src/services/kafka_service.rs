use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::events::types::AgentTrustEvent;

#[derive(Clone)]
pub struct KafkaProducer {
    inner: Option<FutureProducer>,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .ok();
        if producer.is_none() {
            tracing::warn!("Failed to create Kafka producer — events will be disabled");
        }
        Self { inner: producer }
    }

    pub fn disabled() -> Self {
        Self { inner: None }
    }

    /// Publish an event to Kafka. Best-effort: failures are logged but not propagated.
    pub async fn publish(&self, topic: &str, event: &AgentTrustEvent) {
        let Some(producer) = &self.inner else {
            return;
        };
        let key = event.aggregate_id.clone();
        let payload = match serde_json::to_string(event) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to serialize Kafka event: {}", e);
                return;
            }
        };
        let record = FutureRecord::to(topic).key(&key).payload(&payload);
        match producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => tracing::debug!("Published event {} to {}", event.event_type, topic),
            Err((e, _)) => tracing::warn!("Failed to publish to Kafka topic {}: {}", topic, e),
        }
    }

    /// Convenience: create and publish an event.
    pub async fn send(
        &self,
        topic: &str,
        event_type: &str,
        aggregate_id: &str,
        payload: serde_json::Value,
    ) {
        let event = AgentTrustEvent::new(event_type, aggregate_id, payload, "agenttrust-server");
        self.publish(topic, &event).await;
    }
}
