use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use crate::models::flow_policy::FlowPolicy;

#[derive(Debug, Serialize)]
pub struct FlowViolation {
    pub rule: String,
    pub current: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize)]
pub enum FlowDecision {
    Allow,
    Deny { violations: Vec<FlowViolation> },
}

pub async fn check(
    db: &PgPool,
    redis: &mut ConnectionManager,
    sender_did: &str,
    receiver_did: &str,
) -> AppResult<FlowDecision> {
    let policy = get_policy(db, sender_did).await?;
    let mut violations = Vec::new();

    // Rule 1: per-minute rate
    let minute_key = format!("flow:{}:minute", sender_did);
    let minute_count: i64 = redis::cmd("GET")
        .arg(&minute_key)
        .query_async(redis)
        .await
        .unwrap_or(0i64);
    if minute_count >= policy.max_transactions_per_minute as i64 {
        violations.push(FlowViolation {
            rule: "max_per_minute".into(),
            current: minute_count,
            limit: policy.max_transactions_per_minute as i64,
        });
    }

    // Rule 2: per-hour rate
    let hour_key = format!("flow:{}:hour", sender_did);
    let hour_count: i64 = redis::cmd("GET")
        .arg(&hour_key)
        .query_async(redis)
        .await
        .unwrap_or(0i64);
    if hour_count >= policy.max_transactions_per_hour as i64 {
        violations.push(FlowViolation {
            rule: "max_per_hour".into(),
            current: hour_count,
            limit: policy.max_transactions_per_hour as i64,
        });
    }

    // Rule 3: same-pair daily
    let pair_key = format!("flow:{}:{}:daily", sender_did, receiver_did);
    let pair_count: i64 = redis::cmd("GET")
        .arg(&pair_key)
        .query_async(redis)
        .await
        .unwrap_or(0i64);
    if pair_count >= policy.max_a2a_with_same_agent_per_day as i64 {
        violations.push(FlowViolation {
            rule: "max_same_pair_daily".into(),
            current: pair_count,
            limit: policy.max_a2a_with_same_agent_per_day as i64,
        });
    }

    // Rule 4: chain depth (BFS)
    let chain_depth = get_chain_depth(db, sender_did, receiver_did).await?;
    if chain_depth >= policy.max_chain_depth as i64 {
        violations.push(FlowViolation {
            rule: "max_chain_depth".into(),
            current: chain_depth,
            limit: policy.max_chain_depth as i64,
        });
    }

    // Rule 5: active sagas
    let active_row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM sagas WHERE status IN ('started','in_progress') AND a2a_transfer_id IN (SELECT id FROM a2a_transfers WHERE sender_did = $1)"
    )
    .bind(sender_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let active_sagas: i64 = {
        use sqlx::Row;
        active_row.try_get("cnt").unwrap_or(0)
    };
    if active_sagas >= 5 {
        violations.push(FlowViolation {
            rule: "max_active_sagas".into(),
            current: active_sagas,
            limit: 5,
        });
    }

    if violations.is_empty() {
        // Increment counters (best-effort)
        let _ = increment_flow_counter(redis, &minute_key, 60).await;
        let _ = increment_flow_counter(redis, &hour_key, 3600).await;
        let _ = increment_flow_counter(redis, &pair_key, 86400).await;
        Ok(FlowDecision::Allow)
    } else {
        Ok(FlowDecision::Deny { violations })
    }
}

async fn increment_flow_counter(
    redis: &mut ConnectionManager,
    key: &str,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let _: i64 = redis::cmd("INCR").arg(key).query_async(redis).await?;
    let _: bool = redis::cmd("EXPIRE").arg(key).arg(ttl_secs).query_async(redis).await?;
    Ok(())
}

async fn get_chain_depth(db: &PgPool, sender_did: &str, receiver_did: &str) -> AppResult<i64> {
    let rows = sqlx::query(
        r#"
        SELECT sender_did, receiver_did
        FROM a2a_transfers
        WHERE status NOT IN ('refunded', 'timeout')
          AND initiated_at > NOW() - INTERVAL '1 hour'
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    use sqlx::Row;
    let transfers: Vec<(String, String)> = rows
        .iter()
        .map(|r| {
            let s: String = r.try_get("sender_did").unwrap_or_default();
            let recv: String = r.try_get("receiver_did").unwrap_or_default();
            (s, recv)
        })
        .collect();

    // BFS from receiver to find if it reaches sender
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((receiver_did.to_string(), 0i64));
    visited.insert(receiver_did.to_string());

    while let Some((current, depth)) = queue.pop_front() {
        if current == sender_did {
            return Ok(depth);
        }
        for (s, r) in &transfers {
            if s == &current && !visited.contains(r) {
                visited.insert(r.clone());
                queue.push_back((r.clone(), depth + 1));
            }
        }
    }
    Ok(0)
}

pub async fn get_policy(db: &PgPool, agent_did: &str) -> AppResult<FlowPolicy> {
    // Try agent-specific policy first, then global
    let row = sqlx::query_as::<_, FlowPolicy>(
        "SELECT * FROM flow_policies WHERE (agent_did = $1 OR agent_did IS NULL) AND is_active = true ORDER BY agent_did NULLS LAST LIMIT 1",
    )
    .bind(agent_did)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(row.unwrap_or_else(default_policy))
}

fn default_policy() -> FlowPolicy {
    FlowPolicy {
        id: uuid::Uuid::nil(),
        agent_did: None,
        max_transactions_per_minute: 10,
        max_transactions_per_hour: 100,
        max_transactions_per_day: 1000,
        max_a2a_with_same_agent_per_day: 10,
        max_chain_depth: 5,
        max_saga_timeout_minutes: 60,
        max_escrow_timeout_hours: 24,
        auto_freeze_on_consecutive_failures: 10,
        auto_freeze_on_daily_amount_exceed: None,
        is_active: true,
        created_at: None,
        updated_at: None,
    }
}

pub async fn configure(db: &PgPool, req: ConfigureRequest) -> AppResult<FlowPolicy> {
    let id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO flow_policies (id, agent_did, max_transactions_per_minute, max_transactions_per_hour,
            max_a2a_with_same_agent_per_day, max_chain_depth, max_saga_timeout_minutes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(&req.agent_did)
    .bind(req.max_transactions_per_minute.unwrap_or(10))
    .bind(req.max_transactions_per_hour.unwrap_or(100))
    .bind(req.max_a2a_with_same_agent_per_day.unwrap_or(10))
    .bind(req.max_chain_depth.unwrap_or(5))
    .bind(req.max_saga_timeout_minutes.unwrap_or(60))
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    get_policy(db, req.agent_did.as_deref().unwrap_or("")).await
}

pub async fn get_health(db: &PgPool, redis: &mut ConnectionManager, agent_did: &str) -> AppResult<FlowHealth> {
    let policy = get_policy(db, agent_did).await?;

    let minute_count: i64 = redis::cmd("GET")
        .arg(format!("flow:{}:minute", agent_did))
        .query_async(redis)
        .await
        .unwrap_or(0);
    let hour_count: i64 = redis::cmd("GET")
        .arg(format!("flow:{}:hour", agent_did))
        .query_async(redis)
        .await
        .unwrap_or(0);

    let active_sagas: i64 = {
        use sqlx::Row;
        let r = sqlx::query(
            "SELECT COUNT(*) as cnt FROM sagas WHERE status IN ('started','in_progress') AND a2a_transfer_id IN (SELECT id FROM a2a_transfers WHERE sender_did = $1)"
        )
        .bind(agent_did)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        r.try_get("cnt").unwrap_or(0)
    };

    let active_escrows: i64 = {
        use sqlx::Row;
        let r = sqlx::query(
            "SELECT COUNT(*) as cnt FROM escrows WHERE payer_did = $1 AND status = 'funded'"
        )
        .bind(agent_did)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        r.try_get("cnt").unwrap_or(0)
    };

    let health = if minute_count < policy.max_transactions_per_minute as i64
        && hour_count < policy.max_transactions_per_hour as i64
    {
        "healthy"
    } else {
        "degraded"
    };

    Ok(FlowHealth {
        agent_did: agent_did.to_string(),
        current_minute_transactions: minute_count,
        current_hour_transactions: hour_count,
        active_sagas,
        active_escrows,
        policy,
        health: health.to_string(),
        warnings: vec![],
    })
}

#[derive(Debug, Deserialize)]
pub struct ConfigureRequest {
    pub agent_did: Option<String>,
    pub max_transactions_per_minute: Option<i32>,
    pub max_transactions_per_hour: Option<i32>,
    pub max_a2a_with_same_agent_per_day: Option<i32>,
    pub max_chain_depth: Option<i32>,
    pub max_saga_timeout_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct FlowHealth {
    pub agent_did: String,
    pub current_minute_transactions: i64,
    pub current_hour_transactions: i64,
    pub active_sagas: i64,
    pub active_escrows: i64,
    pub policy: FlowPolicy,
    pub health: String,
    pub warnings: Vec<String>,
}
