use tokio_postgres::Row;
use crate::postgres::model::entity::DeliveryEntity;
use crate::postgres::model::result::RepositoryError;

pub fn row_to_delivery_entity(row: &Row) -> Result<DeliveryEntity, RepositoryError> {
    Ok(
        DeliveryEntity {
            delivery_id: map_row(row, 0, "delivery_id")?,
            order_id: map_row(row, 1, "order_id")?,
            address: map_row(row, 2, "address")?,
            status: map_row(row, 3, "status")?,
            created_at: map_row(row, 4, "created_at")?,
            updated_at: map_row(row, 5, "updated_at")?
        }
    )
}

fn map_row<T>(row: &Row, idx: usize, column_name: &str) -> Result<T, RepositoryError>
where
    T: for<'a> tokio_postgres::types::FromSql<'a>,
{
    row.try_get(idx)
        .map_err(|e| RepositoryError::ParseError(format!("{}: {}", column_name, e)))
}

