use crate::tools::email::Error;
use quick_xml::DeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error(transparent)]
    DeserializationError(#[from] DeError),
    #[error(transparent)]
    MailError(#[from] Error),
    #[error("Missing events incoming addresses")]
    MissingEventsIncomingAddresses,
    #[error(transparent)]
    TeraInitializationError(#[from] tera::Error),
}
