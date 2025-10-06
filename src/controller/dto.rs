use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDeliveryRequest {
    pub order_id: Option<String>,
    pub address: Option<String>,
    pub items: Option<Vec<OrderItemDto>>
}

#[derive(Serialize)]
pub struct DeliveryResponse {
    pub delivery_id: String,
    pub order_id: String,
    pub address: String,
    pub status: String,
    pub items: Vec<OrderItemDto>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct OrderItemDto {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Serialize)]
pub struct ErrorResponseDto {
    pub error: &'static str,
    pub message: String
}