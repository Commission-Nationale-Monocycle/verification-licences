use std::ffi::OsStr;

use crate::error::ApplicationError;
use crate::membership::error::MembershipError;
use crate::membership::file_details::FileDetails;
use crate::membership::import_from_file::{find_file, import_from_file};
use crate::membership::indexed_memberships::IndexedMemberships;
use crate::tools::log_message;
use derive_getters::Getters;

type Result<T, E = ApplicationError> = std::result::Result<T, E>;

#[derive(Debug, Getters, Default, Eq, PartialEq)]
pub struct MembershipsState {
    file_details: Option<FileDetails>,
    memberships: IndexedMemberships,
}

impl MembershipsState {
    pub fn new(file_details: Option<FileDetails>, memberships: IndexedMemberships) -> Self {
        Self {
            file_details,
            memberships,
        }
    }

    pub fn set_file_details(&mut self, file_details: FileDetails) {
        self.file_details = Some(file_details);
    }

    pub fn set_memberships(&mut self, memberships: IndexedMemberships) {
        self.memberships = memberships;
    }

    fn load_memberships_file_details(
        memberships_file_folder: &OsStr,
    ) -> Result<Option<FileDetails>> {
        match find_file(memberships_file_folder) {
            Ok(file_details) => Ok(Some(file_details)),
            Err(ApplicationError::Membership(MembershipError::NoFileFound)) => Ok(None),
            Err(e) => {
                log_message("Can't read members file.")(&e);
                Err(e)
            }
        }
    }

    /// Look for a file containing members and load said file into memory.
    pub fn load_memberships(memberships_file_folder: &OsStr) -> Result<MembershipsState> {
        let file_details = match Self::load_memberships_file_details(memberships_file_folder) {
            Err(e) => return Err(e),
            Ok(None) => return Ok(MembershipsState::default()),
            Ok(Some(details)) => details,
        };

        let members = import_from_file(file_details.filepath())?;
        let memberships_state = MembershipsState::new(Some(file_details), members);
        Ok(memberships_state)
    }
}

#[cfg(test)]
mod tests {
    mod load_memberships_file_details {
        use crate::error::ApplicationError::Membership;
        use crate::membership::error::MembershipError::CantBrowseThroughFiles;
        use crate::tools::test::tests::temp_dir;
        use crate::web::api::memberships_state::MembershipsState;
        use chrono::NaiveDate;
        use std::fs::File;

        #[test]
        fn success() {
            let year = 2025;
            let month = 2;
            let day = 1;
            let temp_dir = temp_dir();
            let members_file = temp_dir.join(format!("memberships-{year}-{month:02}-{day:02}.csv"));
            File::create(&members_file).unwrap();

            let file_details =
                MembershipsState::load_memberships_file_details(&temp_dir.into_os_string())
                    .unwrap()
                    .unwrap();
            assert_eq!(&members_file.into_os_string(), file_details.filepath());
            assert_eq!(
                &NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                file_details.update_date()
            );
        }

        #[test]
        fn fail_when_no_file_found() {
            let temp_dir = temp_dir();

            let file_details =
                MembershipsState::load_memberships_file_details(&temp_dir.into_os_string())
                    .unwrap();
            assert_eq!(None, file_details);
        }

        #[test]
        fn fail_when_path_is_file_and_not_folder() {
            let year = 2025;
            let month = 2;
            let day = 1;
            let temp_dir = temp_dir();
            let members_file = temp_dir.join(format!("memberships-{year}-{month:02}-{day:02}.csv"));
            File::create(&members_file).unwrap();

            let error =
                MembershipsState::load_memberships_file_details(&members_file.into_os_string())
                    .err()
                    .unwrap();
            assert!(matches!(error, Membership(CantBrowseThroughFiles(_))));
        }
    }

    mod load_memberships {
        use std::fs;

        use crate::error::ApplicationError::Membership;
        use crate::membership::error::MembershipError::CantBrowseThroughFiles;
        use crate::membership::file_details::FileDetails;
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::tools::test::tests::temp_dir;
        use crate::web::api::memberships_state::MembershipsState;
        use chrono::NaiveDate;
        use dto::membership::tests::{get_expected_membership, get_membership_as_csv};

        #[test]
        fn should_load_members() {
            let year = 2025;
            let month = 2;
            let day = 1;
            let temp_dir = temp_dir();
            let memberships_file =
                temp_dir.join(format!("memberships-{year}-{month:02}-{day:02}.csv"));
            fs::write(&memberships_file, get_membership_as_csv()).unwrap();

            let state = MembershipsState::load_memberships(&temp_dir.into_os_string()).unwrap();
            assert_eq!(
                MembershipsState::new(
                    Some(FileDetails::new(
                        NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                        memberships_file.into_os_string()
                    )),
                    IndexedMemberships::from(vec![get_expected_membership()])
                ),
                state
            );
        }

        #[test]
        fn should_not_load_members_when_no_file() {
            let temp_dir = temp_dir();

            let state = MembershipsState::load_memberships(&temp_dir.into_os_string()).unwrap();
            assert_eq!(MembershipsState::default(), state);
        }

        #[test]
        fn should_not_load_members_when_error() {
            let year = 2025;
            let month = 2;
            let day = 1;
            let temp_dir = temp_dir();
            let members_file = temp_dir.join(format!("memberships-{year}-{month:02}-{day:02}.csv"));
            fs::write(&members_file, get_membership_as_csv()).unwrap();

            let result = MembershipsState::load_memberships(&members_file.into_os_string());
            dbg!(&result);
            let error = result.err().unwrap();
            assert!(matches!(error, Membership(CantBrowseThroughFiles(_))));
        }
    }
}
