use std::ffi::OsStr;

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod file_details;
pub(crate) mod grouped_memberships;
pub(crate) mod import_from_file;
pub(crate) mod memberships;

#[cfg(not(feature = "demo"))]
const MEMBERSHIPS_FILE_FOLDER: &str = "data";
#[cfg(feature = "demo")]
const MEMBERSHIPS_FILE_FOLDER: &str = "demo_data";

/// Retrieve the default memberships file folder name, which depends on the mode (demo or release).
pub fn get_memberships_file_folder() -> &'static OsStr {
    MEMBERSHIPS_FILE_FOLDER.as_ref()
}
