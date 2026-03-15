use sqlx::PgPool;
use crate::services::{escrow_service, kafka_service::KafkaProducer, saga_service};

pub async fn check_timed_out_sagas(db: &PgPool, kafka: &KafkaProducer) {
    let timed_out = match saga_service::get_timed_out(db).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to fetch timed-out sagas: {}", e);
            return;
        }
    };

    for saga in timed_out {
        tracing::warn!("Saga {} timed out. Starting compensation.", saga.id);
        // Find associated transfer's escrow
        let escrow_id = sqlx::query("SELECT escrow_id FROM a2a_transfers WHERE saga_id = $1")
            .bind(saga.id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .and_then(|r| {
                use sqlx::Row;
                r.try_get::<Option<uuid::Uuid>, _>("escrow_id").ok().flatten()
            });

        if let Err(e) = saga_service::compensate(db, kafka, saga.id, saga.current_step, escrow_id, kafka, db).await {
            tracing::error!("Failed to compensate timed-out saga {}: {}", saga.id, e);
        }

        // Update transfer status to timeout
        if let Err(e) = sqlx::query(
            "UPDATE a2a_transfers SET status = 'timeout', updated_at = NOW() WHERE saga_id = $1"
        )
        .bind(saga.id)
        .execute(db)
        .await {
            tracing::error!("Failed to update transfer status for saga {}: {}", saga.id, e);
        }
    }
}
