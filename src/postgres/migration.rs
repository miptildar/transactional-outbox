use crate::postgres::connection::PgConnectionPool;
use refinery::embed_migrations;
use std::ops::DerefMut;

embed_migrations!("./migrations");

pub struct MigrationRunner;

impl MigrationRunner {
    pub async fn run_migrations(
        pool: &PgConnectionPool
    ) -> anyhow::Result<()> {
        let mut client = pool.get_connection().await?;

        let result = migrations::runner()
            .run_async(&mut **client)
            .await;

        if let Err(error) = result {
            panic!("Failed to apply migrations: {}", error);
        }

        Ok(())
    }
}