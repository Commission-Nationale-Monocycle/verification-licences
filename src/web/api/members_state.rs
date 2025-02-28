use std::ffi::OsStr;
use derive_getters::Getters;
use crate::member::error::Error;
use crate::member::file_details::FileDetails;
use crate::member::import_from_file::{find_file, import_from_file};
use crate::member::members::Members;
use crate::tools::log_message;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Getters, Default, Eq, PartialEq)]
pub struct MembersState {
    file_details: Option<FileDetails>,
    members: Members,
}

impl MembersState {
    pub fn new(file_details: Option<FileDetails>, members: Members) -> Self {
        Self { file_details, members }
    }

    pub fn set_file_details(&mut self, file_details: FileDetails) {
        self.file_details = Some(file_details);
    }

    pub fn set_members(&mut self, members: Members) {
        self.members = members;
    }

    fn load_members_file_details(members_file_folder: &OsStr) -> Result<Option<FileDetails>> {
        match find_file(members_file_folder) {
            Ok(file_details) => Ok(Some(file_details)),
            Err(Error::NoFileFound) => Ok(None),
            Err(e) => {
                log_message("Can't read members file.")(&e);
                Err(e)
            },
        }
    }

    /// Look for a file containing members and load said file into memory.
    pub fn load_members(members_file_folder: &OsStr) -> Result<MembersState> {
        let file_details = match Self::load_members_file_details(members_file_folder) {
            Err(e) => return Err(e),
            Ok(None) => return Ok(MembersState::default()),
            Ok(Some(details)) => details,
        };

        let members = import_from_file(file_details.filepath())?;
        let members_state = MembersState::new(Some(file_details), members);
        Ok(members_state)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};
    use std::fs;
    use std::fs::File;
    use chrono::NaiveDate;
    use crate::member::error::Error::CantBrowseThroughFiles;
    use crate::member::file_details::FileDetails;
    use crate::member::members::Members;
    use crate::member::tests::{get_expected_member, get_member_as_csv, MEMBERSHIP_NUMBER};
    use crate::tools::test::tests::temp_dir;
    use crate::web::api::members_state::MembersState;

    // region load_members_file_details
    #[test]
    fn should_load_members_file_details() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.csv"));
        File::create(&members_file).unwrap();

        let file_details = MembersState::load_members_file_details(&temp_dir.into_os_string()).unwrap().unwrap();
        assert_eq!(&members_file.into_os_string(), file_details.filepath());
        assert_eq!(&NaiveDate::from_ymd_opt(year, month, day).unwrap(), file_details.update_date());
    }

    #[test]
    fn should_not_load_members_file_details_when_no_file_found() {
        let temp_dir = temp_dir();

        let file_details = MembersState::load_members_file_details(&temp_dir.into_os_string()).unwrap();
        assert_eq!(None, file_details);
    }

    #[test]
    fn should_not_load_members_file_details_when_path_is_file_and_not_folder() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.csv"));
        File::create(&members_file).unwrap();

        let error = MembersState::load_members_file_details(&members_file.into_os_string())
            .err()
            .unwrap();
        assert_eq!(CantBrowseThroughFiles, error);
    }
    // endregion

    // region load_members
    #[test]
    fn should_load_members() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.csv"));
        fs::write(&members_file, get_member_as_csv()).unwrap();

        let state = MembersState::load_members(&temp_dir.into_os_string()).unwrap();
        assert_eq!(MembersState::new(
            Some(FileDetails::new(NaiveDate::from_ymd_opt(year, month, day).unwrap(), members_file.into_os_string())),
            Members::from(HashMap::from([(MEMBERSHIP_NUMBER.to_owned(), BTreeSet::from([get_expected_member()]))]))
        ), state);
    }

    #[test]
    fn should_not_load_members_when_no_file() {
        let temp_dir = temp_dir();

        let state = MembersState::load_members(&temp_dir.into_os_string()).unwrap();
        assert_eq!(MembersState::default(), state);
    }

    #[test]
    fn should_not_load_members_when_error() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.csv"));
        fs::write(&members_file, get_member_as_csv()).unwrap();

        let result = MembersState::load_members(&members_file.into_os_string());
        dbg!(&result);
        let error = result
            .err()
            .unwrap();
        assert_eq!(CantBrowseThroughFiles, error);
    }
    // endregion
}
