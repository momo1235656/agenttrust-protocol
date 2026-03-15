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
use crate::services::{trust_service, vc_service};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trust/:agent_did/score", get(get_score))
        .route("/trust/:agent_did/history", get(get_history))
        .route("/trust/:agent_did/recalculate", post(recalculate))
}

async fn get_score(
    State(state): State<Arc<AppState>>,
    Path(agent_did): Path<String>,
) -> AppResult<impl IntoResponse> {
    let did = urlencoding::decode(&agent_did)
        .map(|s| s.into_owned())
        .unwrap_or(agent_did);

    let score = trust_service::get_latest_score(&state.db, &did)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("No trust score found".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "agent_did": score.agent_did,
            "score": score.score,
            "risk_level": vc_service::risk_level(score.score),
            "breakdown": {
                "total_transactions": score.total_transactions,
                "successful_transactions": score.successful_transactions,
                "failed_transactions": score.failed_transactions,
                "success_rate": score.success_rate,
                "dispute_count": score.dispute_count,
                "dispute_rate": score.dispute_rate,
                "total_volume": score.total_volume,
                "avg_transaction_value": score.avg_transaction_value,
                "unique_counterparties": score.unique_counterparties,
                "account_age_days": score.account_age_days
            },
            "calculation_version": score.calculation_version,
            "calculated_at": score.calculated_at
        })),
    ))
}

#[derive(Deserialize)]
struct HistoryQuery {
    from: Option<String>,
    to: Option<String>,
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(agent_did): Path<String>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> AppResult<impl IntoResponse> {
    let did = urlencoding::decode(&agent_did)
        .map(|s| s.into_owned())
        .unwrap_or(agent_did);

    let from = query.from.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let to = query.to.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let history = trust_service::get_score_history(&state.db, &did, from, to).await?;

    let history_json: Vec<_> = history
        .iter()
        .map(|s| serde_json::json!({ "timestamp": s.calculated_at, "score": s.score }))
        .collect();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "agent_did": did,
            "history": history_json
        })),
    ))
}

async fn recalculate(
    State(state): State<Arc<AppState>>,
    Path(agent_did): Path<String>,
) -> AppResult<impl IntoResponse> {
    let did = urlencoding::decode(&agent_did)
        .map(|s| s.into_owned())
        .unwrap_or(agent_did);

    let previous = trust_service::get_latest_score(&state.db, &did)
        .await?
        .map(|s| s.score)
        .unwrap_or(50);

    let new_score = trust_service::recalculate_score(&state.db, &did).await?;
    let delta = new_score.score - previous;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "agent_did": did,
            "previous_score": previous,
            "new_score": new_score.score,
            "delta": delta,
            "calculated_at": new_score.calculated_at
        })),
    ))
}
