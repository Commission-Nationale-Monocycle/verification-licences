use crate::tools::error::Error::CantParseSelector;
use scraper::error::SelectorErrorKind;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum Error {
    CantCreateClient,
    CantCreateMembershipsFileFolder,
    WrongEncoding,

    WrongCredentials,
    ConnectionFailed,
    CantLoadListOnServer,
    CantRetrieveDownloadLink,
    CantReadPageContent,
    NoDownloadLink,
    FileNotFoundOnServer,
    CantReadMembersDownloadResponse,
    CantWriteMembersFile,
    CantAccessOrganizationMemberships,
    CantParseSelector(String),
    LackOfPermissions,

    CantOpenMembersFileFolder,
    CantOpenMembersFile,
    WrongRegex,
    CantBrowseThroughFiles,
    CantConvertDateFieldToString,
    NoFileFound,

    InvalidDate,
}

impl From<SelectorErrorKind<'_>> for Error {
    fn from(value: SelectorErrorKind<'_>) -> Self {
        CantParseSelector(value.to_string())
    }
}
