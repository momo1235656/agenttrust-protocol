use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::services::kafka_service::KafkaProducer;

pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Config,
    pub stripe_circuit_breaker: Arc<CircuitBreaker>,
    pub kafka: KafkaProducer,
}

impl AppState {
    pub fn new(db: PgPool, redis: ConnectionManager, config: Config, kafka: KafkaProducer) -> Self {
        Self {
            db,
            redis,
            config,
            stripe_circuit_breaker: Arc::new(CircuitBreaker::new(5, 60)),
            kafka,
        }
    }
}

pub type SharedState = Arc<AppState>;
