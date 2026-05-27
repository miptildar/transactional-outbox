use crate::kafka::topics::DELIVERY_TOPIC;
use crate::postgres::model::delivery::Delivery;
use crate::postgres::model::outbox::{OutboxAggregateType, OutboxMessage, OutboxStatus};
use crate::postgres::model::result::RepositoryError;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use tokio_postgres::{Row, Transaction};
use uuid::Uuid;

// Payload события — что получат потребители из Kafka
#[derive(Serialize)]
struct DeliveryCreatedPayload<'a> {
    delivery_id: Uuid,
    order_id: &'a str,
    status: &'a str,
    recipient_name: &'a str,
    recipient_phone: &'a str,
    city: &'a str,
    scheduled_date: NaiveDate,
    items: Vec<DeliveryItemPayload<'a>>,
}

#[derive(Serialize)]
struct DeliveryItemPayload<'a> {
    sku: &'a str,
    name: &'a str,
    quantity: u32,
}

const INSERT_NEW_OUTBOX_TASK: &'static str = "\
    INSERT INTO outbox (\
        id, aggregate_type, topic, key, payload, status, \
        processing_attempts, next_retry_at, last_error, processed_at, \
        created_at, updated_at) \
    VALUES ($1, $2, $3, $4, $5, 'NEW', 0, NULL, NULL, NULL, $6, $7)
";

const FETCH_PENDING_TASKS: &'static str = "\
    SELECT \
        id, aggregate_type, topic, key, payload, status, \
        processing_attempts, next_retry_at, last_error, processed_at, \
        created_at, updated_at \
    FROM outbox \
    WHERE status IN ('NEW', 'WAITING_RETRY') \
    AND (next_retry_at IS NULL OR next_retry_at <= now()) \
    ORDER BY created_at\
    LIMIT $1 \
    FOR UPDATE SKIP LOCKED
";

const SCHEDULE_RETRY: &'static str = "\
    UPDATE outbox SET \
        status = 'WAITING_RETRY', \
        processing_attempts = processing_attempts + 1, \
        next_retry_at = $2, \
        last_error = $3, \
        updated_at = now()
    WHERE id = $1
";

const MARK_TASK_FAILED: &'static str = "\
    UPDATE outbox SET
        status = 'FAILED',
        last_error = $2,
        updated_at = now()
    WHERE id = $1
";

const MARK_TASK_PROCESSED: &'static str = "\
    UPDATE outbox SET
        status = 'PROCESSED',
        processed_at = now(),
        updated_at = now()
    WHERE id = $1
";

pub struct OutboxRepository;

impl OutboxRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn save(
        &self,
        tx: &Transaction<'_>,
        delivery: &Delivery,
    ) -> Result<(), RepositoryError> {
        let key = delivery.order_id.clone();
        let topic = DELIVERY_TOPIC;
        let payload_json = self.create_delivery_payload(&delivery)?;
        let now = Utc::now();

        tx.execute(
            INSERT_NEW_OUTBOX_TASK,
            &[
                &Uuid::new_v4(),
                &OutboxAggregateType::Delivery.as_str(),
                &topic,
                &key,
                &payload_json,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| RepositoryError::DatabaseError(e))
        .map(|_| ())
    }

    pub async fn fetch_pending_tasks(
        &self,
        tx: &Transaction<'_>,
        limit: i32,
    ) -> Result<Vec<OutboxMessage>, RepositoryError> {
        let rows = tx.query(FETCH_PENDING_TASKS, &[&limit]).await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            let message = Self::row_to_outbox_message(row)?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn row_to_outbox_message(row: &Row) -> Result<OutboxMessage, RepositoryError> {
        let aggregate_type_str: String = row.try_get(outbox_columns::AGGREGATE_TYPE)?;
        let aggregate_type = OutboxAggregateType::try_from(aggregate_type_str.as_str())
            .map_err(|e| RepositoryError::ParseError(e))?;

        let status_str: String = row.try_get(outbox_columns::STATUS)?;
        let status = OutboxStatus::try_from(status_str.as_str())
            .map_err(|e| RepositoryError::ParseError(e))?;

        Ok(OutboxMessage {
            id: row.try_get(outbox_columns::ID)?,
            aggregate_type,
            topic: row.try_get(outbox_columns::TOPIC)?,
            key: row.try_get(outbox_columns::KEY)?,
            status,
            payload: row.try_get(outbox_columns::PAYLOAD)?,
            processing_attempts: row.try_get::<_, i32>(outbox_columns::PROCESSING_ATTEMPTS)? as u8,
            next_retry_at: row.try_get(outbox_columns::NEXT_RETRY_AT)?,
            last_error: row.try_get(outbox_columns::LAST_ERROR)?,
            processed_at: row.try_get(outbox_columns::PROCESSED_AT)?,
            created_at: row.try_get(outbox_columns::CREATED_AT)?,
            updated_at: row.try_get(outbox_columns::UPDATED_AT)?,
        })
    }

    pub async fn schedule_retry(
        &self,
        client: &tokio_postgres::Client,
        id: Uuid,
        next_retry_at: DateTime<Utc>,
        error_message: &str,
    ) -> Result<(), RepositoryError> {
        client
            .execute(SCHEDULE_RETRY, &[&id, &next_retry_at, &error_message])
            .await
            .map_err(|e| RepositoryError::DatabaseError(e))
            .map(|_| ())
    }

    pub async fn mark_task_processed(
        &self,
        client: &tokio_postgres::Client,
        id: Uuid,
    ) -> Result<(), RepositoryError> {
        client
            .execute(MARK_TASK_PROCESSED, &[&id])
            .await
            .map_err(|e| RepositoryError::DatabaseError(e))
            .map(|_| ())
    }

    pub async fn mark_task_failed(
        &self,
        client: &tokio_postgres::Client,
        id: Uuid,
        error_message: &str,
    ) -> Result<(), RepositoryError> {
        client
            .execute(MARK_TASK_FAILED, &[&id, &error_message])
            .await
            .map_err(|e| RepositoryError::DatabaseError(e))
            .map(|_| ())
    }

    fn create_delivery_payload(
        &self,
        delivery: &Delivery,
    ) -> Result<serde_json::Value, RepositoryError> {
        let payload = DeliveryCreatedPayload {
            delivery_id: delivery.id,
            order_id: &delivery.order_id,
            status: delivery.status.as_str(),
            recipient_name: &delivery.recipient.name,
            recipient_phone: &delivery.recipient.phone,
            city: &delivery.address.city,
            scheduled_date: delivery.scheduled_date,
            items: delivery
                .items
                .iter()
                .map(|item| DeliveryItemPayload {
                    sku: &item.sku,
                    name: &item.name,
                    quantity: item.quantity,
                })
                .collect(),
        };

        serde_json::to_value(payload).map_err(|e| RepositoryError::ParseError(e.to_string()))
    }
}

mod outbox_columns {
    pub const ID: &str = "id";
    pub const AGGREGATE_TYPE: &str = "aggregate_type";
    pub const TOPIC: &str = "topic";
    pub const KEY: &str = "key";
    pub const PAYLOAD: &str = "payload";
    pub const STATUS: &str = "status";
    pub const PROCESSING_ATTEMPTS: &str = "processing_attempts";
    pub const NEXT_RETRY_AT: &str = "next_retry_at";
    pub const LAST_ERROR: &str = "last_error";
    pub const PROCESSED_AT: &str = "processed_at";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
}
