use chrono::Utc;
use sqlx::{PgPool, Row};

use crate::error::{AppError, AppResult};
use crate::models::trust_score::TrustScore;

pub struct AgentMetrics {
    pub total_transactions: i64,
    pub successful_transactions: i64,
    pub failed_transactions: i64,
    pub success_rate: f64,
    pub dispute_count: i64,
    pub dispute_rate: f64,
    pub total_volume: i64,
    pub avg_transaction_value: i64,
    pub unique_counterparties: i64,
    pub account_age_days: i32,
}

pub fn calculate_trust_score(metrics: &AgentMetrics) -> i16 {
    let success_score = metrics.success_rate;
    let dispute_score = 1.0_f64 - (metrics.dispute_rate * 100.0).min(1.0);
    let volume_score = (metrics.total_transactions as f64 / 1000.0).min(1.0);
    let diversity_score = (metrics.unique_counterparties as f64 / 50.0).min(1.0);
    let age_score = (metrics.account_age_days as f64 / 365.0).min(1.0);

    let raw_score = success_score * 0.30
        + dispute_score * 0.25
        + volume_score * 0.15
        + diversity_score * 0.15
        + age_score * 0.15;

    let variable = (raw_score * 50.0) as i16;
    (50 + variable).min(100)
}

pub async fn get_agent_metrics(db: &PgPool, agent_did: &str) -> AppResult<AgentMetrics> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'succeeded') as successful,
            COUNT(*) FILTER (WHERE status = 'failed') as failed,
            COALESCE(SUM(amount) FILTER (WHERE status = 'succeeded'), 0) as total_volume,
            COALESCE(AVG(amount) FILTER (WHERE status = 'succeeded'), 0) as avg_value,
            COUNT(DISTINCT description) as unique_counterparties
        FROM transactions
        WHERE agent_did = $1
        "#,
    )
    .bind(agent_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let total: i64 = row.try_get("total").unwrap_or(0);
    let successful: i64 = row.try_get("successful").unwrap_or(0);
    let failed: i64 = row.try_get("failed").unwrap_or(0);
    let total_volume: i64 = row.try_get("total_volume").unwrap_or(0);
    let avg_value: f64 = row.try_get("avg_value").unwrap_or(0.0);
    let unique_counterparties: i64 = row.try_get("unique_counterparties").unwrap_or(0);

    let success_rate = if total > 0 { successful as f64 / total as f64 } else { 1.0 };
    let dispute_rate = 0.0_f64;

    let age_row = sqlx::query(
        "SELECT EXTRACT(DAY FROM NOW() - created_at)::INTEGER as age_days FROM agents WHERE did = $1",
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let account_age_days: i32 = age_row
        .and_then(|r| r.try_get::<Option<i32>, _>("age_days").ok().flatten())
        .unwrap_or(0);

    Ok(AgentMetrics {
        total_transactions: total,
        successful_transactions: successful,
        failed_transactions: failed,
        success_rate,
        dispute_count: 0,
        dispute_rate,
        total_volume,
        avg_transaction_value: avg_value as i64,
        unique_counterparties,
        account_age_days,
    })
}

pub async fn recalculate_score(db: &PgPool, agent_did: &str) -> AppResult<TrustScore> {
    let metrics = get_agent_metrics(db, agent_did).await?;
    let score = calculate_trust_score(&metrics);

    sqlx::query(
        r#"
        INSERT INTO trust_scores (
            agent_did, score, total_transactions, successful_transactions,
            failed_transactions, success_rate, dispute_count, dispute_rate,
            total_volume, avg_transaction_value, unique_counterparties,
            account_age_days, calculation_version, calculated_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,'v1.0',NOW())
        "#,
    )
    .bind(agent_did)
    .bind(score)
    .bind(metrics.total_transactions)
    .bind(metrics.successful_transactions)
    .bind(metrics.failed_transactions)
    .bind(metrics.success_rate)
    .bind(metrics.dispute_count)
    .bind(metrics.dispute_rate)
    .bind(metrics.total_volume)
    .bind(metrics.avg_transaction_value)
    .bind(metrics.unique_counterparties)
    .bind(metrics.account_age_days)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    get_latest_score(db, agent_did)
        .await?
        .ok_or_else(|| AppError::Internal("Score not found after insert".to_string()))
}

pub async fn get_latest_score(db: &PgPool, agent_did: &str) -> AppResult<Option<TrustScore>> {
    let row = sqlx::query_as::<_, TrustScore>(
        r#"
        SELECT agent_did, score, total_transactions, successful_transactions,
               failed_transactions, success_rate, dispute_count, dispute_rate,
               total_volume, avg_transaction_value, unique_counterparties,
               account_age_days, calculation_version, calculated_at
        FROM trust_scores
        WHERE agent_did = $1
        ORDER BY calculated_at DESC
        LIMIT 1
        "#,
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(row)
}

pub async fn get_score_history(
    db: &PgPool,
    agent_did: &str,
    from: Option<chrono::DateTime<Utc>>,
    to: Option<chrono::DateTime<Utc>>,
) -> AppResult<Vec<TrustScore>> {
    let from = from.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let to = to.unwrap_or_else(|| Utc::now());

    let rows = sqlx::query_as::<_, TrustScore>(
        r#"
        SELECT agent_did, score, total_transactions, successful_transactions,
               failed_transactions, success_rate, dispute_count, dispute_rate,
               total_volume, avg_transaction_value, unique_counterparties,
               account_age_days, calculation_version, calculated_at
        FROM trust_scores
        WHERE agent_did = $1
          AND calculated_at >= $2
          AND calculated_at <= $3
        ORDER BY calculated_at ASC
        "#,
    )
    .bind(agent_did)
    .bind(from)
    .bind(to)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(rows)
}
