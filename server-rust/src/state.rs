use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;
use crate::middleware::circuit_breaker::CircuitBreaker;

pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Config,
    pub stripe_circuit_breaker: Arc<CircuitBreaker>,
}

impl AppState {
    pub fn new(db: PgPool, redis: ConnectionManager, config: Config) -> Self {
        Self {
            db,
            redis,
            config,
            stripe_circuit_breaker: Arc::new(CircuitBreaker::new(5, 60)),
        }
    }
}

pub type SharedState = Arc<AppState>;
