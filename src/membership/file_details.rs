use chrono::NaiveDate;
use derive_getters::Getters;
use std::ffi::OsString;

#[derive(Debug, Default, Getters, Eq, PartialEq)]
pub struct FileDetails {
    update_date: NaiveDate,
    filepath: OsString,
}

impl FileDetails {
    pub fn new(update_date: NaiveDate, filepath: OsString) -> Self {
        Self {
            update_date,
            filepath,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::membership::file_details::FileDetails;
    use chrono::NaiveDate;
    use std::ffi::OsString;

    #[test]
    fn test_new_file_details() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let filepath = OsString::from("file");
        let details = FileDetails::new(date, filepath.clone());

        assert_eq!(&date, details.update_date());
        assert_eq!(&filepath, details.filepath());
    }
}
