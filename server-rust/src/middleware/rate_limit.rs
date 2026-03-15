use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

use crate::error::AppError;
use crate::services::rate_limiter::RateLimiter;
use crate::state::SharedState;

/// IP-based rate limiting middleware.
pub async fn ip_rate_limit_middleware(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = addr.ip().to_string();
    let limiter = RateLimiter::new(
        state.config.rate_limit_per_agent_per_minute,
        state.config.rate_limit_per_ip_per_minute,
    );

    let mut redis = state.redis.clone();
    limiter.check_ip_limit(&mut redis, &ip).await?;

    Ok(next.run(request).await)
}
