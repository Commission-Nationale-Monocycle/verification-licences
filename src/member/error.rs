#[derive(Debug)]
pub enum Error {
    CantCreateMembersFileFolder,

    NoCredentials,
    ConnectionFailed,
    ConnectionFailedBecauseOfServer,
    CantLoadListOnServer,
    CantPrepareListForExport,
    CantExportList,

    CantOpenMembersFileFolder,
    CantOpenMembersFile,
    WrongRegex,
    CantBrowseThroughFiles,
    CantConvertDateFieldToString,
    NoFileFound,
}