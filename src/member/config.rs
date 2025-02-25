use std::ffi::OsString;
use derive_getters::Getters;
use regex::Regex;

#[derive(Getters)]
pub struct MembersProviderConfig {
    host: String,
    download_link_regex: Regex,
    folder: OsString,
}

impl MembersProviderConfig {
    pub fn new(host: String, download_link_regex: Regex, folder: OsString) -> Self {
        Self { host, download_link_regex, folder }
    }
}