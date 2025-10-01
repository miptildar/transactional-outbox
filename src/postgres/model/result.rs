#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
    #[error("Parse error: {0}")]
    ParseError(String),
}