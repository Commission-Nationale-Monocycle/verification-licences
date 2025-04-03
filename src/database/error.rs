use crate::database::error::DatabaseError::{ConversionError, UnderlyingDatabase};
use chrono::ParseError;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DatabaseError {
    #[error("The --database-url argument is missing.")]
    MissingDatabaseUrl,
    #[error("An error with r2d2 has occurred.")]
    R2d2(String),
    #[error("The connection to the database failed.")]
    ConnectionFailed,
    #[error("An error occurred within the database.")]
    UnderlyingDatabase(String),
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
    #[error("Can't convert from database value")]
    ConversionError(String),
    #[error("Can't update last updated field")]
    CantUpdateLastUpdated(String),
    #[error("Last update should be known at this point.")]
    UnknownLastUpdate,
}

impl From<Box<dyn Error + Send + Sync + 'static>> for DatabaseError {
    fn from(value: Box<dyn Error + Send + Sync + 'static>) -> Self {
        UnderlyingDatabase(value.to_string())
    }
}

impl From<ParseError> for DatabaseError {
    fn from(value: ParseError) -> Self {
        ConversionError(value.to_string())
    }
}
