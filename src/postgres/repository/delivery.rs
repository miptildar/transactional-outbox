use crate::postgres::model::entity::DeliveryEntity;
use crate::postgres::model::result::RepositoryError;
use crate::postgres::repository::mapper::row_to_delivery_entity;
use chrono::Utc;
use deadpool_postgres::Client;
use tokio_postgres::Transaction;

const INSERT_DELIVERY_ENTITY: &'static str =
    "INSERT INTO deliveries (id, order_id, address, status) \
    VALUES ($1, $2, $3, $4) \
    RETURNING id, order_id, address, status, created_at, updated_at
    ";
const FIND_DELIVERY_BY_ID: &'static str =
    "SELECT id, order_id, address, status, created_at, updated_at FROM deliveries WHERE id = $1";

pub struct DeliveryRepository;

impl DeliveryRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn save(
        &self,
        tx: &Transaction<'_>,
        entity: &DeliveryEntity,
    ) -> Result<DeliveryEntity, RepositoryError> {
        let row = tx
            .query_one(
                INSERT_DELIVERY_ENTITY,
                &[
                    &entity.delivery_id,
                    &entity.order_id,
                    &entity.address,
                    &entity.status,
                    &Utc::now().to_string(),
                    &Utc::now().to_string()
                ],
            )
            .await?;

        Ok(row_to_delivery_entity(&row)?)
    }

    pub async fn find_by_id(
        &self,
        client: Client,
        id: &str,
    ) -> Result<Option<DeliveryEntity>, RepositoryError> {
        let rows = client.query(FIND_DELIVERY_BY_ID, &[&id]).await?;

        if let Some(row) = rows.first() {
            Ok(Some(row_to_delivery_entity(row)?))
        } else {
            Ok(None)
        }
    }
}
