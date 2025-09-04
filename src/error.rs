use crate::database::error::DatabaseError;
use crate::fileo::error::FileoError;
use crate::notification::error::NotificationError;
use crate::web::error::WebError;
use thiserror::Error;
use uda_connector::error::UdaError;

pub type Result<T, E = ApplicationError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("An error has occurred with the database.")]
    Database(#[from] DatabaseError),
    #[error("An error has been encountered while executing requests onto another server.")]
    Web(#[from] WebError),
    #[error("Error while working with Fileo.")]
    Fileo(#[from] FileoError),
    #[error("Error while working with UDA.")]
    Uda(#[from] UdaError),
    #[error("Error while notifying of incoming events.")]
    Notification(#[from] NotificationError),
}
