#[derive(Debug, PartialEq)]
pub enum Error {
    CantCreateClient,
    CantCreateMembershipsFileFolder,
    WrongEncoding,
    NoCredentials,

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
