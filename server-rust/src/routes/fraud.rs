use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppResult;
use crate::services::fraud_service::{self, FraudCheckInput};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/fraud/check", post(check))
        .route("/fraud/:agent_did/alerts", get(alerts))
}

#[derive(Deserialize)]
struct CheckRequest {
    agent_did: String,
    amount: i64,
    description: Option<String>,
    transaction_id: Option<String>,
}

async fn check(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CheckRequest>,
) -> AppResult<impl IntoResponse> {
    let input = FraudCheckInput {
        agent_did: body.agent_did,
        amount: body.amount,
        description: body.description.unwrap_or_default(),
        transaction_id: body.transaction_id,
    };
    let result = fraud_service::check_transaction(&state.db, &input).await?;
    Ok((StatusCode::OK, Json(result)))
}

async fn alerts(
    State(state): State<Arc<AppState>>,
    Path(agent_did): Path<String>,
) -> AppResult<impl IntoResponse> {
    let did = urlencoding::decode(&agent_did)
        .map(|s| s.into_owned())
        .unwrap_or(agent_did);

    let alerts = fraud_service::get_alerts(&state.db, &did).await?;
    let open_alerts = alerts.iter().filter(|a| a.status == "open").count();
    let total = alerts.len();

    Ok((StatusCode::OK, Json(serde_json::json!({
        "agent_did": did,
        "alerts": alerts,
        "total_alerts": total,
        "open_alerts": open_alerts
    }))))
}
