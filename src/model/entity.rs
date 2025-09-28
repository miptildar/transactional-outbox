use chrono::{DateTime, Utc};

pub struct DeliveryEntity {
    pub delivery_id: String,
    pub order_id: String,
    pub address: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum OutboxMessageType {
    Delivery,
}

pub enum OutboxMessageStatus {
    New,
    Processed,
    WaitingRetry,
    Failed,
}

pub struct OutboxMessageEntity {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub topic: String,
    pub key: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_attempts: u8,
    pub last_error: Option<String>,
}
