use std::ffi::OsStr;

use dto::membership::Membership;

pub mod config;
pub mod error;
pub mod file_details;
pub mod import_from_file;
pub mod members;
pub mod memberships;

#[cfg(not(feature = "demo"))]
const MEMBERSHIPS_FILE_FOLDER: &str = "data";
#[cfg(feature = "demo")]
const MEMBERSHIPS_FILE_FOLDER: &str = "demo_data";

/// Retrieve the default memberships file folder name, which depends on the mode (demo or release).
pub fn get_memberships_file_folder() -> &'static OsStr {
    MEMBERSHIPS_FILE_FOLDER.as_ref()
}
