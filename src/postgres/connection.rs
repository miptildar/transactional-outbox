use crate::config::app_config::PostgresEnvConfig;
use deadpool_postgres::{Config, Manager, Object, Pool, PoolError, Runtime};
use tokio_postgres::{ NoTls};
use crate::postgres::migration::migrations;

pub struct PgConnectionPool {
    pool: Pool,
}

impl PgConnectionPool {
    pub async fn initialize(&mut self, environment_config: &PostgresEnvConfig) -> Result<(), deadpool_postgres::CreatePoolError> {
        let mut pool_config = Config::new();
        pool_config.host = Some(environment_config.host.clone());
        pool_config.dbname = Some(environment_config.db_name.clone());
        pool_config.user = Some(environment_config.username.clone());
        pool_config.password = Some(environment_config.password.clone());
        pool_config.pool = Some(deadpool_postgres::PoolConfig::new(environment_config.max_size));

        self.pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls)?;
        Ok(())
    }

    pub async fn get_connection(&self) -> Result<Object, PoolError> {
        self.pool.get().await
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        let mut client = self.get_connection().await?;

        migrations::runner()
            .run_async(&mut **client)
            .await?;

        Ok(())
    }
}

