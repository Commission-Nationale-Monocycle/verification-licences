use crate::fileo::error::FileoError::WrongEncoding;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileoError {
    #[error("Error while trying to create folder for storing memberships file.")]
    MembershipsFileFolderCreationFailed(std::io::Error),
    #[error("The memberships file has an unexpected encoding [error: {0}]")]
    WrongEncoding(Cow<'static, str>),
    #[error("Fileo server couldn't load the requested list.")]
    CantLoadListOnServer,
    #[error("Can't retrieve the memberships file download link from Fileo.")]
    CantRetrieveDownloadLink,
    #[error("The memberships file download link doesn't seem to appear in the page.")]
    NoDownloadLink,
    #[error("The memberships file can't be read as bytes.")]
    MalformedMembershipsDownloadResponse,
    #[error("Can't save the memberships file.")]
    MembershipsFileWriteFailed(std::io::Error),
}

impl From<Cow<'static, str>> for FileoError {
    fn from(value: Cow<'static, str>) -> Self {
        WrongEncoding(value)
    }
}
