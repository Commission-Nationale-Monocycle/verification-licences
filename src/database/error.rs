use crate::database::error::DatabaseError::UnderlyingDatabaseError;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DatabaseError {
    #[error("The --database-url argument is missing.")]
    MissingDatabaseUrl,
    #[error("The connection to the database failed.")]
    ConnectionFailed,
    #[error("An error occurred within the database.")]
    UnderlyingDatabaseError(String),
}

impl From<Box<dyn Error + Send + Sync + 'static>> for DatabaseError {
    fn from(value: Box<dyn Error + Send + Sync + 'static>) -> Self {
        UnderlyingDatabaseError(value.to_string())
    }
}
