use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::postgres::model::delivery::{Address, Delivery, DeliveryItem, Recipient};

#[derive(Deserialize)]
pub struct CreateDeliveryRequest {
    pub order_id: String,
    pub recipient: RecipientDto,
    pub address: AddressDto,
    pub scheduled_date: NaiveDate,
    pub items: Vec<CreateDeliveryItemDto>,
}

#[derive(Serialize, Deserialize)]
pub struct RecipientDto {
    pub name: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddressDto {
    pub city: String,
    pub street: String,
    pub building: String,
    pub apartment: Option<String>,
    pub postal_code: String,
}

#[derive(Deserialize)]
pub struct CreateDeliveryItemDto {
    pub sku: String,
    pub name: String,
    pub quantity: u32,
    pub weight_grams: Option<u32>,
}

#[derive(Serialize)]
pub struct DeliveryResponse {
    pub id: Uuid,
    pub order_id: String,
    pub courier_id: Option<Uuid>,
    pub recipient: RecipientDto,
    pub address: AddressDto,
    pub status: String,
    pub scheduled_date: NaiveDate,
    pub delivered_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub items: Vec<DeliveryItemDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DeliveryItemDto {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: u32,
    pub weight_grams: Option<u32>,
}

impl From<&Delivery> for DeliveryResponse {
    fn from(value: &Delivery) -> Self {
        DeliveryResponse {
            id: value.id,
            order_id: value.order_id.clone(),
            courier_id: value.courier_id,
            recipient: RecipientDto::from(&value.recipient),
            address:  AddressDto::from(&value.address),
            status: value.status.as_str().to_string(),
            scheduled_date: value.scheduled_date,
            delivered_at: value.delivered_at,
            cancelled_at: value.cancelled_at,
            cancellation_reason: value.cancellation_reason.clone(),
            items: value.items.iter().map(DeliveryItemDto::from).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<&Address> for AddressDto {
    fn from(value: &Address) -> Self {
        AddressDto {
            city: value.city.clone(),
            street: value.street.clone(),
            building: value.building.clone(),
            apartment: value.apartment.clone(),
            postal_code: value.postal_code.clone(),
        }
    }
}

impl From<&Recipient> for RecipientDto {
    fn from(r: &Recipient) -> Self {
        RecipientDto {
            name: r.name.clone(),
            phone: r.phone.clone(),
        }
    }
}

impl From<&DeliveryItem> for DeliveryItemDto {
    fn from(item: &DeliveryItem) -> Self {
        DeliveryItemDto {
            id: item.id,
            sku: item.sku.clone(),
            name: item.name.clone(),
            quantity: item.quantity,
            weight_grams: item.weight_grams,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponseDto {
    pub error: &'static str,
    pub message: String
}