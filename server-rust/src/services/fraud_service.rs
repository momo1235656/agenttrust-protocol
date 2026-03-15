use chrono::{Timelike, Utc};
use sqlx::{PgPool, Row};

use crate::error::{AppError, AppResult};

#[derive(Debug)]
pub struct FraudCheckInput {
    pub agent_did: String,
    pub amount: i64,
    pub description: String,
    pub transaction_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct FraudCheckResult {
    pub risk_score: f64,
    pub risk_level: String,
    pub decision: String,
    pub triggered_rules: Vec<String>,
    pub details: serde_json::Value,
}

pub async fn check_transaction(
    db: &PgPool,
    input: &FraudCheckInput,
) -> AppResult<FraudCheckResult> {
    let mut risk_score: f64 = 0.0;
    let mut triggered_rules: Vec<String> = Vec::new();

    let avg_row = sqlx::query(
        r#"
        SELECT COALESCE(AVG(amount), 0)::BIGINT as avg_amount,
               COUNT(*) as tx_count
        FROM transactions
        WHERE agent_did = $1
          AND created_at >= NOW() - INTERVAL '30 days'
          AND status = 'succeeded'
        "#,
    )
    .bind(&input.agent_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let avg_amount: i64 = avg_row.try_get("avg_amount").unwrap_or(0_i64).max(1);
    let monthly_count: i64 = avg_row.try_get("tx_count").unwrap_or(0);
    let ratio = input.amount as f64 / avg_amount as f64;

    if ratio >= 10.0 {
        risk_score += 0.40;
        triggered_rules.push("amount_10x_average".to_string());
    } else if ratio >= 5.0 {
        risk_score += 0.20;
        triggered_rules.push("amount_5x_average".to_string());
    }

    let hourly_row = sqlx::query(
        r#"
        SELECT COUNT(*) as cnt
        FROM transactions
        WHERE agent_did = $1
          AND created_at >= NOW() - INTERVAL '1 hour'
        "#,
    )
    .bind(&input.agent_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let hourly_count: i64 = hourly_row.try_get("cnt").unwrap_or(0);

    let normal_hourly = (monthly_count as f64 / 720.0).max(1.0);
    if hourly_count as f64 >= normal_hourly * 5.0 {
        risk_score += 0.30;
        triggered_rules.push("frequency_5x_hourly".to_string());
    }

    let hour = Utc::now().hour();
    let is_night = hour < 5;
    if is_night && input.amount >= 50_000 {
        risk_score += 0.25;
        triggered_rules.push("night_high_value".to_string());
    }

    let rapid_row = sqlx::query(
        r#"
        SELECT COUNT(*) as cnt
        FROM transactions
        WHERE agent_did = $1
          AND created_at >= NOW() - INTERVAL '5 minutes'
        "#,
    )
    .bind(&input.agent_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let rapid_count: i64 = rapid_row.try_get("cnt").unwrap_or(0);
    if rapid_count >= 3 {
        risk_score += 0.35;
        triggered_rules.push("rapid_succession".to_string());
    }

    risk_score = risk_score.min(1.0);

    let risk_level = match risk_score {
        s if s < 0.3 => "low",
        s if s < 0.7 => "medium",
        _ => "high",
    };

    let decision = match risk_score {
        s if s < 0.3 => "allow",
        s if s < 0.7 => "review",
        _ => "block",
    };

    if decision == "block" {
        let severity = if risk_score >= 0.9 { "critical" } else { "high" };
        for rule in &triggered_rules {
            let details = serde_json::json!({
                "amount": input.amount,
                "avg_amount": avg_amount,
                "ratio": ratio,
                "hourly_count": hourly_count,
                "rapid_count": rapid_count
            });
            sqlx::query(
                r#"
                INSERT INTO fraud_alerts (
                    agent_did, transaction_id, alert_type, severity,
                    risk_score, rule_name, details
                ) VALUES ($1,$2,$3,$4,$5,$6,$7)
                "#,
            )
            .bind(&input.agent_did)
            .bind(input.transaction_id.as_deref())
            .bind(rule.as_str())
            .bind(severity)
            .bind(risk_score)
            .bind(rule.as_str())
            .bind(details)
            .execute(db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    let details = serde_json::json!({
        "amount_vs_average": ratio,
        "hourly_frequency": hourly_count,
        "is_night": is_night,
        "recent_rapid_count": rapid_count
    });

    Ok(FraudCheckResult {
        risk_score,
        risk_level: risk_level.to_string(),
        decision: decision.to_string(),
        triggered_rules,
        details,
    })
}

pub async fn get_alerts(db: &PgPool, agent_did: &str) -> AppResult<Vec<crate::models::fraud_alert::FraudAlert>> {
    let alerts = sqlx::query_as::<_, crate::models::fraud_alert::FraudAlert>(
        r#"
        SELECT id, agent_did, transaction_id, alert_type, severity,
               risk_score, rule_name, details, status,
               resolved_at, resolved_by, created_at
        FROM fraud_alerts
        WHERE agent_did = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(agent_did)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(alerts)
}
