mod config;
mod controller;
mod postgres;
mod service;
mod kafka;

use crate::config::app_config::AppConfig;
use crate::controller::delivery_controller::AppState;
use crate::postgres::connection::PgConnectionPool;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tracing_subscriber::EnvFilter;
use postgres::migration;

#[tokio::main]
async fn main() -> () {
    initialize_logging();

    let settings = AppConfig::load()
        .unwrap_or_else(|e| panic!("Config error: {}", e));

    let postgres_connection_pool = initialize_postgres_connection_pool(&settings).await;

    let app_state: Arc<AppState> = Arc::new(AppState {
        delivery_service: Arc::new(service::delivery::DeliveryService::new(Arc::new(
            postgres_connection_pool,
        ))),
    });

    let router = controller::delivery_controller::init_router(app_state.clone());
    start_server(router, &settings).await;
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

async fn initialize_postgres_connection_pool(settings: &AppConfig) -> PgConnectionPool {
    let postgres_connection_pool = PgConnectionPool::new(&settings.postgres).await
        .unwrap_or_else(|e| panic!("Failed to create Postgres connection pool: {}", e));

    migration::MigrationRunner::run_migrations(&postgres_connection_pool).await
        .unwrap_or_else(|e| panic!("Failed to run DB migrations: {}", e));

    postgres_connection_pool
}

async fn start_server(router: Router, settings: &AppConfig) -> () {
    let addr = format!("{}:{}", settings.server.host, settings.server.port)
        .parse::<SocketAddr>()
        .unwrap_or_else(|e| panic!("Invalid socket address: {}", e));

    tracing::info!(address = %addr, "Server started");

    let listener = tokio::net::TcpListener::bind(addr).await
        .unwrap_or_else(|e| panic!("Failed to bind to address: {}", e));

    axum::serve(listener, router).await
        .unwrap_or_else(|e| panic!("Server error: {}", e));
}
