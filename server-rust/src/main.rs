use std::sync::Arc;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use agenttrust_server::{config, routes, services, state::AppState};
use agenttrust_server::services::kafka_service::KafkaProducer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agenttrust_server=info,tower_http=info".parse().unwrap()),
        )
        .init();

    let config = config::Config::from_env()?;

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::info!("Database migrations applied");

    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("Redis connected");

    // Kafka (optional - disabled if KAFKA_BROKERS not set)
    let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_default();
    let kafka = if kafka_brokers.is_empty() {
        tracing::info!("KAFKA_BROKERS not set — Kafka events disabled");
        KafkaProducer::disabled()
    } else {
        tracing::info!("Connecting to Kafka at {}", kafka_brokers);
        KafkaProducer::new(&kafka_brokers)
    };

    let state = Arc::new(AppState::new(db_pool, redis_conn, config.clone(), kafka));

    // Background: expire stale approvals every 5 minutes
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

    // Background: escrow timeout checker every 60s
    {
        let db = state.db.clone();
        let kafka = state.kafka.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                agenttrust_server::scheduler::escrow_timeout::check_expired_escrows(&db, &kafka).await;
            }
        });
    }

    // Background: saga timeout checker every 60s
    {
        let db = state.db.clone();
        let kafka = state.kafka.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                agenttrust_server::scheduler::saga_timeout::check_timed_out_sagas(&db, &kafka).await;
            }
        });
    }

    // gRPC server on port 50052
    {
        let grpc_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = agenttrust_server::grpc::server::serve(grpc_state, 50052).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
    }

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
        .merge(routes::trust::router())
        .merge(routes::vc::router())
        .merge(routes::fraud::router())
        .merge(routes::a2a::router())
        .merge(routes::escrow::router())
        .merge(routes::saga::router())
        .merge(routes::flow::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("AgentTrust Rust server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
