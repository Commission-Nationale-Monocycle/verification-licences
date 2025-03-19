use crate::uda::error::UdaError::MalformedSelector;
use scraper::error::SelectorErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UdaError {
    #[error("The organisation memberships is inaccessible.")]
    OrganizationMembershipsAccessFailed,
    #[error("Provided selector is malformed [selector: {0}]")]
    MalformedSelector(String),
    #[error("The member can't be marked as confirmed [id: {0}]")]
    MemberConfirmationFailed(u16),
    #[error("The exported XLS file is malformed")]
    MalformedXlsFile,
}

impl From<SelectorErrorKind<'_>> for UdaError {
    fn from(value: SelectorErrorKind<'_>) -> Self {
        MalformedSelector(value.to_string())
    }
}
