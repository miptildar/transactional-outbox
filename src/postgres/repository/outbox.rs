use crate::postgres::connection::PgConnectionPool;
use std::sync::Arc;

pub struct OutboxRepository {
    pool: Arc<PgConnectionPool>
}

impl OutboxRepository {
    pub fn new(pool: Arc<PgConnectionPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self) {

    }
}