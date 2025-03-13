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

    CantOpenMembersFileFolder,
    CantOpenMembersFile,
    WrongRegex,
    CantBrowseThroughFiles,
    CantConvertDateFieldToString,
    NoFileFound,

    InvalidDate,
}
