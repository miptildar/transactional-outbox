use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct Address {
    pub city: String,
    pub street: String,
    pub building: String,
    pub apartment: Option<String>,
    pub postal_code: String,
}

#[derive(Debug)]
pub struct Recipient {
    pub name: String,
    pub phone: String,
}

#[derive(Debug)]
pub struct Delivery {
    pub id: Uuid,
    pub order_id: String,
    pub courier_id: Option<Uuid>,
    pub recipient: Recipient,
    pub address: Address,
    pub status: DeliveryStatus,
    pub scheduled_date: NaiveDate,
    pub delivered_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub items: Vec<DeliveryItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct DeliveryItem {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: u32,
    pub weight_grams: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Assigned,
    PickedUp,
    InTransit,
    Delivered,
    Failed,
    Cancelled,
}

impl DeliveryStatus {

    // Allowed state transitions
    pub fn can_transition_to(&self, next: &DeliveryStatus) -> bool {
        matches!(
              (self, next),
              (Self::Pending,   Self::Assigned)   |
              (Self::Pending,   Self::Cancelled)  |
              (Self::Assigned,  Self::PickedUp)   |
              (Self::Assigned,  Self::Cancelled)  |
              (Self::PickedUp,  Self::InTransit)  |
              (Self::PickedUp,  Self::Cancelled)  |
              (Self::InTransit, Self::Delivered)  |
              (Self::InTransit, Self::Failed)     |
              (Self::InTransit, Self::Cancelled)  |
              (Self::Failed,    Self::InTransit)  | // retry
              (Self::Failed,    Self::Cancelled)
          )
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending   => "PENDING",
            Self::Assigned  => "ASSIGNED",
            Self::PickedUp  => "PICKED_UP",
            Self::InTransit => "IN_TRANSIT",
            Self::Delivered => "DELIVERED",
            Self::Failed    => "FAILED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl TryFrom<&str> for DeliveryStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "PENDING"    => Ok(Self::Pending),
            "ASSIGNED"   => Ok(Self::Assigned),
            "PICKED_UP"  => Ok(Self::PickedUp),
            "IN_TRANSIT" => Ok(Self::InTransit),
            "DELIVERED"  => Ok(Self::Delivered),
            "FAILED"     => Ok(Self::Failed),
            "CANCELLED"  => Ok(Self::Cancelled),
            other => Err(format!("Unknown delivery status: {}", other)),
        }
    }
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct DeliveryStatusHistory {
    pub id: Uuid,
    pub delivery_id: Uuid,
    pub status: DeliveryStatus,
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
}
