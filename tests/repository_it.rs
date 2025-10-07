use std::sync::Arc;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use testcontainers::runners::AsyncRunner;
use transactional_outbox::config::app_config::PostgresEnvConfig;
use transactional_outbox::postgres::connection::PgConnectionPool;
use transactional_outbox::postgres::migration;
use transactional_outbox::postgres::repository::delivery::DeliveryRepository;

struct TestContext {
    _container: ContainerAsync<GenericImage>,
    pool: Arc<PgConnectionPool>,
    repository: DeliveryRepository,
}

const POSTGRES_DB_NAME: &str = "testdb";
const POSTGRES_USER: &str = "testuser";
const POSTGRES_PASSWORD: &str = "testpass";

#[tokio::test]
async fn test_with_postgres() {
    let container = GenericImage::new("postgres", "15")
        .with_wait_for(WaitFor::message_on_stderr("database is ready"))
        .with_exposed_port(ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_DB", POSTGRES_DB_NAME)
        .with_env_var("POSTGRES_USER", POSTGRES_USER)
        .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
        .start()
        .await
        .expect("Failed to start container");

    let host_port = container.get_host_port_ipv4(5432).await
        .expect("Failed to get container port");

    let postgres_config = PostgresEnvConfig {
        host: format!("127.0.0.1:{}", host_port),
        db_name: POSTGRES_DB_NAME.to_string(),
        username: POSTGRES_USER.to_string(),
        password: POSTGRES_PASSWORD.to_string(),
        max_size: 10,
    };

    let pool = Arc::new(
        PgConnectionPool::new(&postgres_config)
            .await
            .expect("Failed to create connection pool")
    );

    migration::MigrationRunner::run_migrations(&pool.clone())
        .await
        .expect("Failed to run migrations");
}