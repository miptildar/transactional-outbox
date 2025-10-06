mod config;
mod controller;
mod postgres;
mod service;

use crate::config::app_config::AppEnvConfig;
use crate::controller::delivery_controller::AppState;
use crate::postgres::connection::PgConnectionPool;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tracing_subscriber::EnvFilter;
use postgres::migration;

#[tokio::main]
async fn main() {
    initialize_logging().await;

    let settings = AppEnvConfig::from_env().expect("сonfig error");

    let postgres_connection_pool = initialize_postgres_connection_pool(&settings).await;

    let app_state: Arc<AppState> = Arc::new(AppState {
        delivery_service: Arc::new(service::delivery::DeliveryService::new(Arc::new(
            postgres_connection_pool,
        ))),
    });

    let router = controller::delivery_controller::init_router(app_state.clone());
    start_server(router, &settings).await.unwrap();
}

async fn initialize_logging() {
    tracing_subscriber::fmt()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

async fn initialize_postgres_connection_pool(settings: &AppEnvConfig) -> PgConnectionPool {
    let postgres_connection_pool = PgConnectionPool::new(&settings.postgres).await.unwrap();
    migration::MigrationRunner::run_migrations(&postgres_connection_pool).await.unwrap();

    postgres_connection_pool
}

async fn start_server(router: Router, settings: &AppEnvConfig) -> anyhow::Result<()> {
    let addr = format!("{}:{}", settings.server.host, settings.server.port)
        .parse::<SocketAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid socket address: {}", e))?;

    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| anyhow::anyhow!("Failed to bind to address: {}", e))?;

    axum::serve(listener, router).await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
