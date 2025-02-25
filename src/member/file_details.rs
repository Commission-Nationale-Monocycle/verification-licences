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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use chrono::NaiveDate;
    use crate::member::file_details::FileDetails;

    #[test]
    fn test_new_file_details() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let filename = OsString::from("file");
        let details = FileDetails::new(date, filename.clone());

        assert_eq!(&date, details.update_date());
        assert_eq!(&filename, details.filename());
    }
}