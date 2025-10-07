use crate::kafka::topics::DELIVERY_TOPIC;
use crate::postgres::model::entity::{
    DeliveryEntity, OutboxDeliveryPayload, OutboxMessageStatus, OutboxMessageType,
};
use crate::postgres::model::result::RepositoryError;
use chrono::Utc;
use tokio_postgres::Transaction;
use uuid::Uuid;

const INSERT_OUTBOX_ENTITY: &'static str = "\
    INSERT INTO outbox (id, type, topic, key, payload, status, processing_attempts, last_error, processed_at, created_at, updated_at) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
";
const UPDATE_ENTITY_STATE: &'static str = "\
    UPDATE outbox SET status = $1 WHERE id = $2
";
const INCREASE_PROCESSING_ATTEMPTS: &'static str = "\
    UPDATE outbox SET processing_attempts = processing_attempts + 1 WHERE id = $1
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
        let key = delivery_entity.order_id.clone();
        let topic = DELIVERY_TOPIC;
        let payload_json = self.create_delivery_payload(&delivery_entity)?;
        let processing_attempts: i32 = 0;
        let last_error: Option<&str> = None;
        let processed_at: Option<chrono::DateTime<Utc>> = None;

        tx.execute(
            INSERT_OUTBOX_ENTITY,
            &[
                &Uuid::new_v4().to_string(),
                &OutboxMessageType::Delivery.to_string(),
                &topic,
                &key,
                &payload_json,
                &OutboxMessageStatus::New.to_string(),
                &processing_attempts,
                &last_error,
                &processed_at,
                &Utc::now().to_string(),
                &Utc::now().to_string(),
            ],
        )
        .await
        .map_err(RepositoryError::DatabaseError)
        .map(|_| ())
    }

    pub async fn update_status(
        &self,
        client: &tokio_postgres::Client,
        id: &str,
        status: &str,
    ) -> Result<(), RepositoryError> {
        client
            .execute(UPDATE_ENTITY_STATE, &[&status, &id])
            .await
            .map_err(|e| RepositoryError::DatabaseError(e))
            .map(|_| ())
    }

    pub async fn increase_processing_attempts(
        &self,
        tx: &Transaction<'_>,
        id: &str,
    ) -> Result<(), RepositoryError> {
        tx.execute(INCREASE_PROCESSING_ATTEMPTS, &[&id])
            .await
            .map_err(|e| RepositoryError::DatabaseError(e))
            .map(|_| ())
    }

    fn create_delivery_payload(
        &self,
        delivery_entity: &DeliveryEntity,
    ) -> Result<String, RepositoryError> {
        let payload = OutboxDeliveryPayload {
            delivery_id: delivery_entity.delivery_id.clone(),
            order_id: delivery_entity.order_id.clone(),
            status: delivery_entity.status.clone(),
        };

        serde_json::to_string(&payload).map_err(|e| RepositoryError::ParseError(e.to_string()))
    }
}
