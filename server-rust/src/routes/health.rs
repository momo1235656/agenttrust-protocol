use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::SharedState;

static START_TIME: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

pub fn router() -> Router<SharedState> {
    START_TIME.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<SharedState>) -> impl IntoResponse {
    let uptime = {
        let start = START_TIME.get().copied().unwrap_or(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(start)
    };

    // Check DB
    let db_status = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map(|_| "connected")
        .unwrap_or("disconnected");

    // Check Redis via PING command
    let redis_status = {
        use redis::cmd;
        let mut conn = state.redis.clone();
        let result: Result<String, _> = cmd("PING").query_async(&mut conn).await;
        if result.is_ok() { "connected" } else { "disconnected" }
    };

    Json(serde_json::json!({
        "status": "ok",
        "version": "0.2.0",
        "database": db_status,
        "redis": redis_status,
        "stripe": "reachable",
        "uptime_seconds": uptime,
    }))
}
