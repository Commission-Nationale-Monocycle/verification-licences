#[derive(Debug)]
pub enum Error {
    NoCredentials,
    ConnectionFailed,
    ConnectionFailedBecauseOfServer,
    CantLoadListOnServer,
    CantPrepareListForExport,
    CantExportList,

    CantOpenMembersFile,
    WrongRegex,
    CantBrowseThroughFiles,
    CantConvertDateFieldToString,
    NoFileFound,

}