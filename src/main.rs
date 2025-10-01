mod config;
mod controller;
mod postgres;
mod service;

use crate::config::app_config::AppEnvConfig;
use crate::controller::delivery_controller::AppState;
use crate::postgres::connection::PgConnectionPool;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let settings = AppEnvConfig::from_env().expect("сonfig error");

    let postgres_connection_pool = PgConnectionPool::new(&settings.postgres).await.unwrap();

    let app_state: Arc<AppState> = Arc::new(AppState {
        delivery_service: Arc::new(service::delivery::DeliveryService::new(Arc::new(
            postgres_connection_pool,
        ))),
    });

    let app = controller::delivery_controller::init_router(app_state.clone());

    let addr = settings
        .postgres
        .host
        .parse::<SocketAddr>()
        .expect("Invalid socket address");
    println!("Listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
