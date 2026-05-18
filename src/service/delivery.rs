use crate::controller::dto::{CreateDeliveryRequest, DeliveryResponse};
use crate::postgres::connection::PgConnectionPool;
use crate::postgres::model::entity::{DeliveryEntity, DeliveryStatus};
use crate::postgres::repository::delivery::DeliveryRepository;
use crate::postgres::repository::outbox::OutboxRepository;
use crate::service::mapper::entity_to_dto;
use std::sync::Arc;
use uuid::Uuid;

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
        let address = request.address
            .filter(|s| !s.trim().is_empty())
            .ok_or(ServiceError::InvalidDto)?;
        let _ = request.items.ok_or(ServiceError::InvalidDto)?;

        let delivery_entity = DeliveryEntity {
            delivery_id: Uuid::new_v4().to_string(),
            order_id,
            address,
            status: DeliveryStatus::Pending.to_uppercase_string(),
            created_at: None,
            updated_at: None,
        };

        let mut client = self.pool.get_connection().await.map_err(|e| {
            tracing::error!("Failed to get Postgres connection: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let tx = client.transaction().await.map_err(|e| {
            tracing::error!("Failed to start transaction: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let saved = self.delivery_repo.save(&tx, &delivery_entity).await
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

        Ok(entity_to_dto(&saved))
    }

    pub async fn get_delivery_by_id(
        &self,
        delivery_id: &str,
    ) -> Result<DeliveryResponse, ServiceError> {
        let client = self.pool.get_connection().await.map_err(|e| {
            tracing::error!("Failed to get connection: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let result = self.delivery_repo.find_by_id(client, delivery_id).await;

        match result {
            Ok(Some(entity)) => Ok(entity_to_dto(&entity)),
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
