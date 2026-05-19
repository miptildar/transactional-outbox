use uuid::Uuid;

pub struct OutboxMessage {
    pub id: Uuid,
    
}

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