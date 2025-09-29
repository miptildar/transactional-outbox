use crate::model::dto::{CreateDeliveryRequest, CreateDeliveryResponse};
use crate::postgres::connection::PgConnectionPool;
use crate::postgres::repository::delivery_repository::DeliveryRepository;
use crate::postgres::repository::outbox_repository::OutboxRepository;
use std::sync::Arc;
use uuid::Uuid;
use crate::model::entity::{DeliveryEntity, DeliveryStatus};

pub struct DeliveryService {
    delivery_repo: DeliveryRepository,
    outbox_repo: OutboxRepository,
    pool: Arc<PgConnectionPool>,
}

impl DeliveryService {
    pub fn new(pool: Arc<PgConnectionPool>) -> Self {
        Self {
            delivery_repo: DeliveryRepository::new(pool.clone()),
            outbox_repo: OutboxRepository::new(pool.clone()),
            pool,
        }
    }

    pub async fn create_delivery(
        &self,
        request: CreateDeliveryRequest,
    ) -> Result<CreateDeliveryResponse, ServiceError> {
        if (!Self::validate(&request)) {
            return Err(ServiceError::InvalidDto);
        }

        let delivery_entity = DeliveryEntity {
            delivery_id: Uuid::new_v4().to_string(),
            order_id: request.order_id.unwrap().clone(),
            address: request.address.unwrap().clone(),
            status: DeliveryStatus::Pending.to_uppercase_string(),
            created_at: None,
            updated_at: None
        };
        
        self.delivery_repo.save();

        Ok(CreateDeliveryResponse {
            delivery_id: "new-delivery-id".to_string(),
            order_id: request.order_id,
            address: request.address,
            status: "PENDING".to_string(),
            items: request.items,
        })
    }

    fn validate(dto: &CreateDeliveryRequest) -> bool {
        if (dto.address.is_none() || dto.order_id.is_none() || dto.items.is_none()) {
            return false;
        }

        let valid_order_id = dto.order_id.as_ref().unwrap().trim().is_empty();
        let valid_address = dto.address.as_ref().unwrap().trim().is_empty();

        valid_order_id && valid_address
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Delivery not found")]
    NotFound,
    #[error("Invalid DTO")]
    InvalidDto,

    #[error("Invalid customer ID")]
    InvalidCustomerId,
    #[error("Invalid status transition")]
    InvalidStatusTransition,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
