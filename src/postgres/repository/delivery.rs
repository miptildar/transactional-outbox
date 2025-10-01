use crate::postgres::model::entity::DeliveryEntity;
use crate::postgres::connection::PgConnectionPool;
use crate::postgres::model::result::RepositoryError;
use crate::postgres::repository::mapper::row_to_delivery_entity;
use std::sync::Arc;

const INSERT_DELIVERY_ENTITY: &'static str =
    "INSERT INTO deliveries (id, order_id, address, status) \
    VALUES ($1, $2, $3, $4) \
    RETURNING id, order_id, address, status, created_at, updated_at
    ";
const FIND_DELIVERY_BY_ID: &'static str =
    "SELECT id, order_id, address, status, created_at, updated_at FROM deliveries WHERE id = $1";

pub struct DeliveryRepository {
    pool: Arc<PgConnectionPool>,
}

impl DeliveryRepository {
    pub fn new(pool: Arc<PgConnectionPool>) -> Self {
        Self { pool }
    }

    pub async fn save(&self, entity: DeliveryEntity) -> Result<DeliveryEntity, RepositoryError> {
        let client = self.pool.get_connection().await?;

        let row = client
            .query_one(
                INSERT_DELIVERY_ENTITY,
                &[
                    &entity.delivery_id,
                    &entity.order_id,
                    &entity.address,
                    &entity.status,
                ],
            )
            .await?;

        Ok(
            row_to_delivery_entity(&row)?
        )
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DeliveryEntity>, RepositoryError> {
        let client = self.pool.get_connection().await?;

        let rows = client.query(
            FIND_DELIVERY_BY_ID, &[&id]
        ).await?;

        if let Some(row) = rows.first() {
            Ok(Some(row_to_delivery_entity(row)?))
        } else {
            Ok(None)
        }
    }
}
