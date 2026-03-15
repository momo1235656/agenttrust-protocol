use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::{AppError, AppResult};

/// Redis-based sliding window rate limiter.
/// Uses Redis INCR + EXPIRE for simplicity.
pub struct RateLimiter {
    pub per_agent_per_minute: u64,
    pub per_ip_per_minute: u64,
}

impl RateLimiter {
    pub fn new(per_agent: u64, per_ip: u64) -> Self {
        Self {
            per_agent_per_minute: per_agent,
            per_ip_per_minute: per_ip,
        }
    }

    pub async fn check_agent_limit(
        &self,
        redis: &mut ConnectionManager,
        agent_did: &str,
    ) -> AppResult<()> {
        let key = format!("rl:agent:{}:{}", agent_did, current_minute());
        self.check_limit(redis, &key, self.per_agent_per_minute).await
    }

    pub async fn check_ip_limit(
        &self,
        redis: &mut ConnectionManager,
        ip: &str,
    ) -> AppResult<()> {
        let key = format!("rl:ip:{}:{}", ip, current_minute());
        self.check_limit(redis, &key, self.per_ip_per_minute).await
    }

    async fn check_limit(
        &self,
        redis: &mut ConnectionManager,
        key: &str,
        limit: u64,
    ) -> AppResult<()> {
        let count: u64 = redis.incr(key, 1_u64).await.unwrap_or(1);
        if count == 1 {
            // Set expiry for 2 minutes to handle edge cases
            let _: Result<bool, redis::RedisError> = redis.expire(key, 120_i64).await;
        }
        if count > limit {
            return Err(AppError::RateLimitExceeded);
        }
        Ok(())
    }
}

fn current_minute() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs() / 60
}
