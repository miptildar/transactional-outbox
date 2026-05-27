use crate::postgres::model::delivery::{
    Address, Delivery, DeliveryItem, DeliveryStatus, Recipient,
};
use crate::postgres::model::result::RepositoryError;
use deadpool_postgres::Client;
use tokio_postgres::{Row, Transaction};
use uuid::Uuid;

const INSERT_DELIVERY: &str = "\
    INSERT INTO deliveries (\
        id, order_id, \
        recipient_name, recipient_phone, \
        city, street, building, apartment, postal_code, \
        status, \
        scheduled_date, \
        created_at, updated_at) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)";

const INSERT_DELIVERY_ITEMS_BATCH: &str = "INSERT INTO delivery_items (id, delivery_id, sku, name, quantity, weight_grams) \
    SELECT * FROM unnest($1, $2, $3, $4, $5, $6)";

const FIND_DELIVERY_BY_ID: &str = "
      SELECT
          id, order_id, courier_id,
          recipient_name, recipient_phone,
          city, street, building, apartment, postal_code,
          status, scheduled_date,
          delivered_at, cancelled_at, cancellation_reason,
          created_at, updated_at
      FROM deliveries
      WHERE id = $1
  ";

const FIND_ITEMS_BY_DELIVERY_ID: &str = "
      SELECT id, sku, name, quantity, weight_grams
      FROM delivery_items
      WHERE delivery_id = $1
  ";

pub struct DeliveryRepository;

impl DeliveryRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn create(
        &self, 
        tx: &Transaction<'_>, 
        delivery: &Delivery,
    ) -> Result<(), RepositoryError> {
        self.insert_delivery(tx, delivery).await?;
        self.insert_items(tx, delivery.id, &delivery.items).await?;
        Ok(())
    }

    async fn insert_delivery(
        &self,
        tx: &Transaction<'_>,
        delivery: &Delivery,
    ) -> Result<(), RepositoryError> {
        tx.execute(
            INSERT_DELIVERY,
            &[
                &delivery.id,
                &delivery.order_id,
                &delivery.recipient.name,
                &delivery.recipient.phone,
                &delivery.address.city,
                &delivery.address.street,
                &delivery.address.building,
                &delivery.address.apartment,
                &delivery.address.postal_code,
                &delivery.status.as_str(),
                &delivery.scheduled_date,
                &delivery.created_at,
                &delivery.updated_at,
            ],
        )
        .await?;

        Ok(())
    }

    async fn insert_items(
        &self,
        tx: &Transaction<'_>,
        delivery_id: Uuid,
        items: &[DeliveryItem],
    ) -> Result<(), RepositoryError> {
        let ids: Vec<Uuid> = items.iter().map(|i| i.id).collect();
        let delivery_ids: Vec<Uuid> = items.iter().map(|_| delivery_id).collect();
        let skus: Vec<&str> = items.iter().map(|i| i.sku.as_str()).collect();
        let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
        let quantities: Vec<i32> = items.iter().map(|i| i.quantity as i32).collect();
        let weights: Vec<Option<i32>> = items
            .iter()
            .map(|i| i.weight_grams.map(|w| w as i32))
            .collect();

        tx.execute(
            INSERT_DELIVERY_ITEMS_BATCH,
            &[&ids, &delivery_ids, &skus, &names, &quantities, &weights],
        )
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        &self,
        client: &Client,
        id: Uuid,
    ) -> Result<Option<Delivery>, RepositoryError> {
        let maybe_row = client.query_opt(FIND_DELIVERY_BY_ID, &[&id]).await?;

        let Some(row) = maybe_row else {
            return Ok(None);
        };

        let item_rows = client.query(FIND_ITEMS_BY_DELIVERY_ID, &[&id]).await?;

        let items = item_rows
            .iter()
            .map(row_to_delivery_item)
            .collect::<Result<Vec<_>, _>>()?;

        let mut delivery = row_to_delivery(&row)?;
        delivery.items = items;

        Ok(Some(delivery))
    }
}

fn row_to_delivery(row: &Row) -> Result<Delivery, RepositoryError> {
    let status_str: String = row.try_get(delivery_columns::STATUS)?;
    let status = DeliveryStatus::try_from(status_str.as_str())
        .map_err(|e| RepositoryError::ParseError(e))?;

    // todo optional fields
    Ok(Delivery {
        id: row.try_get(delivery_columns::ID)?,
        order_id: row.try_get(delivery_columns::ORDER_ID)?,
        courier_id: row.try_get(delivery_columns::COURIER_ID)?,
        recipient: Recipient {
            name: row.try_get(delivery_columns::RECIPIENT_NAME)?,
            phone: row.try_get(delivery_columns::RECIPIENT_PHONE)?,
        },
        address: Address {
            city: row.try_get(delivery_columns::CITY)?,
            street: row.try_get(delivery_columns::STREET)?,
            building: row.try_get(delivery_columns::BUILDING)?,
            apartment: row.try_get(delivery_columns::APARTMENT)?,
            postal_code: row.try_get(delivery_columns::POSTAL_CODE)?,
        },
        status,
        scheduled_date: row.try_get(delivery_columns::SCHEDULED_DATE)?,
        delivered_at: row.try_get(delivery_columns::DELIVERED_AT)?,
        cancelled_at: row.try_get(delivery_columns::CANCELLED_AT)?,
        cancellation_reason: row.try_get(delivery_columns::CANCELLATION_REASON)?,
        items: vec![],
        created_at: row.try_get(delivery_columns::CREATED_AT)?,
        updated_at: row.try_get(delivery_columns::UPDATED_AT)?,
    })
}

fn row_to_delivery_item(row: &Row) -> Result<DeliveryItem, RepositoryError> {
    Ok(DeliveryItem {
        id: row.try_get(delivery_item_columns::ID)?,
        sku: row.try_get(delivery_item_columns::SKU)?,
        name: row.try_get(delivery_item_columns::NAME)?,
        quantity: row.try_get::<_, i32>(delivery_item_columns::QUANTITY)? as u32,
        weight_grams: row
            .try_get::<_, Option<i32>>(delivery_item_columns::WEIGHT_GRAMS)?
            .map(|w| w as u32),
    })
}

mod delivery_columns {
    pub const ID: &str = "id";
    pub const ORDER_ID: &str = "order_id";
    pub const COURIER_ID: &str = "courier_id";
    pub const RECIPIENT_NAME: &str = "recipient_name";
    pub const RECIPIENT_PHONE: &str = "recipient_phone";
    pub const CITY: &str = "city";
    pub const STREET: &str = "street";
    pub const BUILDING: &str = "building";
    pub const APARTMENT: &str = "apartment";
    pub const POSTAL_CODE: &str = "postal_code";
    pub const STATUS: &str = "status";
    pub const SCHEDULED_DATE: &str = "scheduled_date";
    pub const DELIVERED_AT: &str = "delivered_at";
    pub const CANCELLED_AT: &str = "cancelled_at";
    pub const CANCELLATION_REASON: &str = "cancellation_reason";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
}

mod delivery_item_columns {
    pub const ID: &str = "id";
    pub const DELIVERY_ID: &str = "delivery_id";
    pub const SKU: &str = "sku";
    pub const NAME: &str = "name";
    pub const QUANTITY: &str = "quantity";
    pub const WEIGHT_GRAMS: &str = "weight_grams";
}
