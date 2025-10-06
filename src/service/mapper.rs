use crate::controller::dto::DeliveryResponse;
use crate::postgres::model::entity::DeliveryEntity;

pub fn entity_to_dto(entity: &DeliveryEntity) -> DeliveryResponse {
    DeliveryResponse {
        delivery_id: entity.delivery_id.clone(),
        order_id: entity.order_id.clone(),
        address: entity.address.clone(),
        status: entity.status.clone(),
        items: Vec::new(),
        created_at: entity.created_at.map(|dt| dt.to_rfc3339()),
        updated_at: entity.updated_at.map(|dt| dt.to_rfc3339()),
    }
}