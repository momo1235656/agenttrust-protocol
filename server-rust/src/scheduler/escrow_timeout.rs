use sqlx::PgPool;
use crate::services::{escrow_service, kafka_service::KafkaProducer};

pub async fn check_expired_escrows(db: &PgPool, kafka: &KafkaProducer) {
    let expired = match escrow_service::get_expired_funded(db).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to fetch expired escrows: {}", e);
            return;
        }
    };

    for escrow in expired {
        tracing::warn!("Escrow {} expired. Initiating refund.", escrow.id);
        if let Err(e) = escrow_service::refund(db, kafka, escrow.id, "Escrow timeout").await {
            tracing::error!("Failed to refund expired escrow {}: {}", escrow.id, e);
        }
    }
}
