use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub aggregate_type: OutboxAggregateType,
    pub topic: String,
    pub key: String,
    pub payload: serde_json::Value,
    pub status: OutboxStatus,
    pub processing_attempts: u8,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutboxStatus {
    New,
    Processed,
    WaitingRetry,
    Failed,
}

impl OutboxStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::New => "NEW",
            Self::Processed => "PROCESSED",
            Self::WaitingRetry => "WAITING_RETRY",
            Self::Failed => "FAILED",
        }
    }
}

impl TryFrom<&str> for OutboxStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "NEW" => Ok(Self::New),
            "PROCESSED" => Ok(Self::Processed),
            "WAITING_RETRY" => Ok(Self::WaitingRetry),
            "FAILED" => Ok(Self::Failed),
            other => Err(format!("Unknown outbox status: {}", other)),
        }
    }
}

impl std::fmt::Display for OutboxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutboxAggregateType {
    Delivery,
}

impl OutboxAggregateType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Delivery => "DELIVERY",
        }
    }
}

impl TryFrom<&str> for OutboxAggregateType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DELIVERY" => Ok(Self::Delivery),
            other => Err(format!("Unknown outbox aggregate type: {}", other)),
        }
    }
}

impl std::fmt::Display for OutboxAggregateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}