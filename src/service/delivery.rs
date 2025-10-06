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
        if !Self::validate(&request) {
            return Err(ServiceError::InvalidDto);
        }

        let delivery_entity = DeliveryEntity {
            delivery_id: Uuid::new_v4().to_string(),
            order_id: request.order_id.unwrap().clone(),
            address: request.address.unwrap().clone(),
            status: DeliveryStatus::Pending.to_uppercase_string(),
            created_at: None,
            updated_at: None,
        };

        let client = self.pool.get_connection().await;
        if client.is_err() {
            let error_string = client.err().unwrap().to_string();
            tracing::error!("Failed to get Postgres connection: {}", error_string.clone());
            return Err(ServiceError::DatabaseError(error_string));
        }

        let mut client = client.unwrap();
        let tx = client.transaction().await.map_err(|e| {
            tracing::error!("Failed to start transaction: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;
        
        let result = self.delivery_repo.save(&tx, &delivery_entity).await;
        if result.is_err() {
            let _ = tx.rollback().await;
            let error = result.err().unwrap().to_string();
            tracing::error!("Failed to save delivery entity: {}", error.clone());
            return Err(ServiceError::DatabaseError(error));
        }

        let outbox_result = self.outbox_repo.save(&tx, &delivery_entity).await;
        if outbox_result.is_err() {
            let _ = tx.rollback().await;
            let error = outbox_result.err().unwrap().to_string();
            tracing::error!("Failed to save outbox entity: {}", error.clone());
            return Err(ServiceError::DatabaseError(error));
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        let actual_entity = &result.unwrap();

        Ok(entity_to_dto(&actual_entity))
    }

    pub async fn get_delivery_by_id(
        &self,
        delivery_id: &str,
    ) -> Result<DeliveryResponse, ServiceError> {
        let client = self.pool.get_connection().await;
        if client.is_err() {
            let error = client.err().unwrap().to_string();
            tracing::error!("Failed to get connection: {}", error.clone());
            return Err(ServiceError::DatabaseError(error));
        }

        let result = self.delivery_repo.find_by_id(client.unwrap(), delivery_id).await;
        match result {
            Ok(Some(entity)) => Ok(entity_to_dto(&entity)),
            Ok(None) => Err(ServiceError::NotFound),
            Err(err) => {
                let error = err.to_string();
                tracing::error!("Database error: {}", error.clone());
                Err(ServiceError::DatabaseError(error))
            },
        }
    }

    fn validate(dto: &CreateDeliveryRequest) -> bool {
        if dto.address.is_none() || dto.order_id.is_none() || dto.items.is_none() {
            return false;
        }

        let valid_order_id = !dto.order_id.as_ref().unwrap().trim().is_empty();
        let valid_address = !dto.address.as_ref().unwrap().trim().is_empty();

        valid_order_id && valid_address
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
