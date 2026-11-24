use crate::tools::email::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error),
    #[error(transparent)]
    MailError(#[from] Error),
    #[error("Missing events incoming addresses")]
    MissingEventsIncomingAddresses,
    #[error(transparent)]
    TeraInitializationError(#[from] tera::Error),
}
