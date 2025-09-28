mod model;
mod controller;
mod postgres;
mod config;
mod service;

use std::net::SocketAddr;
use crate::config::app_config::AppEnvConfig;

#[tokio::main]
async fn main() {
    let settings = AppEnvConfig::from_env().expect("сonfig error");

    let app = controller::delivery_controller::init_router();

    let addr = settings.postgres.host.parse::<SocketAddr>().expect("Invalid socket address");
    println!("Listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap();
}
