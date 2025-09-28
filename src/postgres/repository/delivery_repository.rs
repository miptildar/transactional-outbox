use crate::postgres::connection::PgConnectionPool;

pub struct DeliveryRepository<'a> {
    pool: &'a PgConnectionPool
}

impl<'a> DeliveryRepository<'a> {
    pub async fn save() {

    }

    pub async fn get_by_id(id: &String) {

    }
}