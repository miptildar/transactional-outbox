use crate::postgres::model::entity::{
    DeliveryEntity, OutboxDeliveryPayload, OutboxMessageStatus, OutboxMessageType,
};
use crate::postgres::model::result::RepositoryError;
use chrono::Utc;
use tokio_postgres::Transaction;
use uuid::Uuid;

const INSERT_OUTBOX_ENTITY: &'static str = "\
    INSERT INTO outbox (id, type, payload, status, created_at, updated_at) \
    VALUES ($1, $2, $3, $4, $5, $6)
";

pub struct OutboxRepository;

impl OutboxRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn save(
        &self,
        tx: &Transaction<'_>,
        delivery_entity: &DeliveryEntity,
    ) -> Result<(), RepositoryError> {
        let payload_json = self.create_delivery_payload(&delivery_entity);
        tx.execute(
            INSERT_OUTBOX_ENTITY,
            &[
                &Uuid::new_v4().to_string(),
                &OutboxMessageType::Delivery.to_string(),
                &payload_json,
                &OutboxMessageStatus::New.to_string(),
                &Utc::now().to_string(),
                &Utc::now().to_string(),
            ],
        )
        .await
        .unwrap();

        Ok(())
    }

    pub async fn update_status(
        &self,
        client: &tokio_postgres::Client,
        id: &str,
        status: &str,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    fn create_delivery_payload(&self, delivery_entity: &DeliveryEntity) -> String {
        let payload = OutboxDeliveryPayload {
            delivery_id: delivery_entity.delivery_id.clone(),
            order_id: delivery_entity.order_id.clone(),
            status: delivery_entity.status.clone(),
        };

        serde_json::to_string(&payload).unwrap()
    }
}