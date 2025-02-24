use std::ffi::OsString;
use chrono::NaiveDate;
use derive_getters::Getters;

#[derive(Debug, Default, Getters)]
pub struct FileDetails {
    update_date: NaiveDate,
    filename: OsString,
}

impl FileDetails {
    pub fn new(update_date: NaiveDate, filename: OsString) -> Self {
        Self { update_date, filename }
    }
}
