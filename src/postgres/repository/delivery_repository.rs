use crate::postgres::connection::PgConnectionPool;
use std::sync::Arc;

pub struct DeliveryRepository {
    pool: Arc<PgConnectionPool>,
}

impl DeliveryRepository {
    pub fn new(pool: Arc<PgConnectionPool>) -> Self {
        Self { pool }
    }

    pub async fn save() {}

    pub async fn get_by_id(id: &String) {}
}
