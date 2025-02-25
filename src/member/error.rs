#[derive(Debug)]
pub enum Error {
    CantCreateClient,
    CantCreateMembersFileFolder,
    CantCreateMembersFile,
    WrongEncoding,
    NoCredentials,

    ConnectionFailed,
    ConnectionFailedBecauseOfServer,
    CantLoadListOnServer,
    CantPrepareListForExport,
    CantReadPageContent,
    NoDownloadLink,
    CantExportList,
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