use crate::config::app_config::PostgresEnvConfig;
use deadpool_postgres::{Config, Object, Pool, PoolError, Runtime};
use tokio_postgres::NoTls;

pub struct PgConnectionPool {
    pool: Pool,
}

impl PgConnectionPool {

    pub async fn new(environment_config: &PostgresEnvConfig) -> Result<Self, deadpool_postgres::CreatePoolError> {
        let mut pool_config = Config::new();
        pool_config.host = Some(environment_config.host.clone());
        pool_config.dbname = Some(environment_config.db_name.clone());
        pool_config.user = Some(environment_config.username.clone());
        pool_config.password = Some(environment_config.password.clone());
        pool_config.pool = Some(deadpool_postgres::PoolConfig::new(environment_config.max_size));

        let pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls);
        if let Result::Err(error) = pool {
            panic!("Error creating connection pool: {}", error);
        }

        Ok(PgConnectionPool {
            pool: pool?,
        })
    }

    pub async fn get_connection(&self) -> Result<Object, PoolError> {
        self.pool.get().await
    }
}

