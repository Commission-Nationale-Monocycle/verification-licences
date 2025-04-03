use derive_getters::Getters;
use regex::Regex;

#[derive(Getters)]
pub struct MembershipsProviderConfig {
    host: String,
    download_link_regex: Regex,
}

impl MembershipsProviderConfig {
    pub fn new(host: String, download_link_regex: Regex) -> Self {
        Self {
            host,
            download_link_regex,
        }
    }
}
