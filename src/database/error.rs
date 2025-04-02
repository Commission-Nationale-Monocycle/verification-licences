use crate::database::error::DatabaseError::UnderlyingDatabase;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DatabaseError {
    #[error("The --database-url argument is missing.")]
    MissingDatabaseUrl,
    #[error("The connection to the database failed.")]
    ConnectionFailed,
    #[error("An error occurred within the database.")]
    UnderlyingDatabase(String),
}

impl From<Box<dyn Error + Send + Sync + 'static>> for DatabaseError {
    fn from(value: Box<dyn Error + Send + Sync + 'static>) -> Self {
        UnderlyingDatabase(value.to_string())
    }
}
