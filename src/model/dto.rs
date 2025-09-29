use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDeliveryRequest {
    pub order_id: Option<String>,
    pub address: Option<String>,
    pub items: Option<Vec<OrderItemDto>>
}

#[derive(Serialize)]
pub struct CreateDeliveryResponse {
    pub delivery_id: String,
    pub order_id: String,
    pub address: String,
    pub status: String,
    pub items: Vec<OrderItemDto>
}

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct OrderItemDto {
}



#[derive(Serialize)]
pub struct ErrorResponseDto {
    pub error: &'static str,
    pub message: String
}