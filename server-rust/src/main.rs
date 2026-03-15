use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use agenttrust_server::{config, routes, services, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agenttrust_server=info,tower_http=info".parse().unwrap()),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env()?;

    // PostgreSQL connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(config.database_url())
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::info!("Database migrations applied");

    // Redis connection
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("Redis connected");

    // Application state
    let state = Arc::new(AppState::new(db_pool, redis_conn, config.clone()));

    // Background task: expire stale approvals every 5 minutes
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                match services::approval_service::expire_stale_approvals(&db).await {
                    Ok(n) if n > 0 => tracing::info!("Expired {} stale approvals", n),
                    Err(e) => tracing::warn!("Failed to expire approvals: {}", e),
                    _ => {}
                }
            }
        });
    }

    // CORS - in production, restrict CORS_ORIGINS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes::did::router())
        .merge(routes::auth::router())
        .merge(routes::payment::router())
        .merge(routes::audit::router())
        .merge(routes::oauth::router())
        .merge(routes::approval::router())
        .merge(routes::health::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("AgentTrust Rust server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
