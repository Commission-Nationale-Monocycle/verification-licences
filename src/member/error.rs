#[derive(Debug, PartialEq)]
pub enum Error {
    CantCreateClient,
    CantCreateMembersFileFolder,
    WrongEncoding,
    NoCredentials,

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