use derive_getters::Getters;
use regex::Regex;
use std::ffi::OsString;

#[derive(Getters)]
pub struct MembershipsProviderConfig {
    host: String,
    download_link_regex: Regex,
    folder: OsString,
}

impl MembershipsProviderConfig {
    pub fn new(host: String, download_link_regex: Regex, folder: OsString) -> Self {
        Self {
            host,
            download_link_regex,
            folder,
        }
    }
}
