use crate::controller::dto::{CreateDeliveryRequest, DeliveryResponse};
use crate::postgres::connection::PgConnectionPool;
use crate::postgres::repository::delivery::DeliveryRepository;
use crate::postgres::repository::outbox::OutboxRepository;
use std::sync::Arc;
use uuid::Uuid;
use transactional_outbox::postgres::model::delivery::{Address, Delivery};
use crate::postgres::model::delivery::{DeliveryItem, DeliveryStatus, Recipient};

pub struct DeliveryService {
    delivery_repo: DeliveryRepository,
    outbox_repo: OutboxRepository,
    pool: Arc<PgConnectionPool>,
}

impl DeliveryService {
    pub fn new(pool: Arc<PgConnectionPool>) -> Self {
        Self {
            delivery_repo: DeliveryRepository::new(),
            outbox_repo: OutboxRepository::new(),
            pool,
        }
    }

    pub async fn create_delivery(
        &self,
        request: CreateDeliveryRequest,
    ) -> Result<DeliveryResponse, ServiceError> {
        let order_id = request.order_id
            .filter(|s| !s.trim().is_empty())
            .ok_or(ServiceError::InvalidDto)?;


        let address = Address {
            city: request.address.city,
            street: request.address.street,
            building: request.address.building,
            apartment: request.address.apartment,
            postal_code: request.address.postal_code,
        };

        let recipient = Recipient {
            name: request.recipient.name,
            phone: request.recipient.phone,
        };

        let delivery_items = request.items.into_iter()
            .map(|dto| DeliveryItem {
                id: Uuid::new_v4(),
                sku: dto.sku,
                name: dto.name,
                quantity: dto.quantity,
                weight_grams: dto.weight_grams,
            })
            .collect();

        let now = chrono::Utc::now();
        let delivery_entity = Delivery {
            id:  Uuid::new_v4(),
            order_id,
            courier_id: None,
            recipient,
            address,
            status: DeliveryStatus::Pending,
            scheduled_date: request.scheduled_date,
            delivered_at: None,
            cancelled_at: None,
            cancellation_reason: None,
            items: delivery_items,
            created_at: now,
            updated_at: now,
        };

        let mut client = self.pool.get_connection().await.map_err(|e| {
            tracing::error!("Failed to get Postgres connection: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let tx = client.transaction().await.map_err(|e| {
            tracing::error!("Failed to start transaction: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        self.delivery_repo.create(&tx, &delivery_entity).await
            .map_err(|e| {
                tracing::error!("Failed to save delivery: {}", e);
                ServiceError::DatabaseError(e.to_string())
            })?;

        self.outbox_repo.save(&tx, &delivery_entity).await
            .map_err(|e| {
                tracing::error!("Failed to save outbox entity: {}", e);
                ServiceError::DatabaseError(e.to_string())
            })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(DeliveryResponse::from(&delivery_entity))
    }

    pub async fn get_delivery_by_id(
        &self,
        delivery_id: Uuid,
    ) -> Result<DeliveryResponse, ServiceError> {
        let client = self.pool.get_connection().await.map_err(|e| {
            tracing::error!("Failed to get connection: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let result = self.delivery_repo.find_by_id(&client, delivery_id).await;

        match result {
            Ok(Some(entity)) => Ok(DeliveryResponse::from(&entity)),
            Ok(None) => Err(ServiceError::NotFound),
            Err(err) => {
                let error = err.to_string();
                tracing::error!("Database error: {}", error);
                Err(ServiceError::DatabaseError(error))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Delivery not found")]
    NotFound,
    #[error("Invalid DTO")]
    InvalidDto,
    #[error("Invalid status transition")]
    InvalidStatusTransition,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
