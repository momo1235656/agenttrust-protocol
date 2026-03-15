use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::error::AppResult;
use crate::services::flow_service;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/flow/configure", post(configure))
        .route("/flow/:agent_did/health", get(health))
}

async fn configure(
    State(state): State<SharedState>,
    Json(req): Json<flow_service::ConfigureRequest>,
) -> AppResult<Json<crate::models::flow_policy::FlowPolicy>> {
    let policy = flow_service::configure(&state.db, req).await?;
    Ok(Json(policy))
}

async fn health(
    State(state): State<SharedState>,
    Path(agent_did): Path<String>,
) -> AppResult<Json<flow_service::FlowHealth>> {
    let mut redis = state.redis.clone();
    let health = flow_service::get_health(&state.db, &mut redis, &agent_did).await?;
    Ok(Json(health))
}
