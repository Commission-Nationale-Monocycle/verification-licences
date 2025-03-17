use std::ffi::OsString;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MembershipError {
    #[error("Memberships file folder can't be opened [path: {0:?}]")]
    CantOpenMembersFileFolder(OsString),
    #[error("Memberships file can't be opened [path: {0:?}]")]
    CantOpenMembersFile(OsString),
    #[error("Provided regex is malformed [regex: {0}]")]
    WrongRegex(String),
    #[error("Can't browse through files to load memberships file [folder: {0:?}]")]
    CantBrowseThroughFiles(OsString),
    #[error("Can't convert date field to string.")]
    CantConvertDateFieldToString,
    #[error("There's no memberships file.")]
    NoFileFound,
    #[error("The provided fields don't represent a valid date.")]
    InvalidDate,
}
