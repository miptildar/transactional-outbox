use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum_macros::EnumString;

pub struct DeliveryEntity {
    pub delivery_id: String,
    pub order_id: String,
    pub address: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(EnumString)]
pub enum DeliveryStatus {
    Pending, InProgress, Delivered
}

impl DeliveryStatus {
    pub fn to_uppercase_string(&self) -> String {
        self.to_string().clone().to_uppercase()
    }
}

impl Display for DeliveryStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "Pending"),
            DeliveryStatus::InProgress => write!(f, "InProgress"),
            DeliveryStatus::Delivered => write!(f, "Delivered"),
        }
    }
}

pub enum OutboxMessageType {
    Delivery,
}

impl Display for OutboxMessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboxMessageType::Delivery => write!(f, "Delivery"),
        }
    }
}

#[derive(EnumString)]
pub enum OutboxMessageStatus {
    New,
    Processed,
    WaitingRetry,
    Failed,
}

impl Display for OutboxMessageStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboxMessageStatus::New => write!(f, "New"),
            OutboxMessageStatus::Processed => write!(f, "Processed"),
            OutboxMessageStatus::WaitingRetry => write!(f, "WaitingRetry"),
            OutboxMessageStatus::Failed => write!(f, "Failed"),
        }
    }
}

pub struct OutboxMessageEntity {
    pub id: String,
    pub aggregate_type: String,
    pub topic: String,
    pub key: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub processing_attempts: u8,
    pub last_error: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct OutboxDeliveryPayload {
    pub delivery_id: String,
    pub order_id: String,
    pub status: String,
}
