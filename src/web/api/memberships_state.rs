use std::ffi::OsString;

use crate::database::dao::last_update::{UpdatableElement, get_last_update};
use crate::database::dao::membership::retrieve_memberships;
use crate::database::error::DatabaseError::UnknownLastUpdate;
use crate::error::ApplicationError;
use crate::membership::file_details::FileDetails;
use crate::membership::indexed_memberships::IndexedMemberships;
use derive_getters::Getters;
use diesel::SqliteConnection;

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

    /// Load memberships into memory.
    pub fn load_memberships(connection: &mut SqliteConnection) -> Result<MembershipsState> {
        let memberships = retrieve_memberships(connection)?;
        let last_update = get_last_update(connection, &UpdatableElement::Memberships)?
            .ok_or_else(|| UnknownLastUpdate)?;
        let file_details = FileDetails::new(last_update.date(), OsString::new());
        let memberships_state = MembershipsState::new(Some(file_details), memberships.into());
        Ok(memberships_state)
    }
}

#[cfg(test)]
mod tests {
    mod load_memberships {
        use std::ffi::OsString;

        use crate::database::dao::membership::replace_memberships;
        use crate::database::with_temp_database;
        use crate::membership::file_details::FileDetails;
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
        use crate::web::api::memberships_state::MembershipsState;
        use chrono::Utc;

        #[test]
        fn should_load_members() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let expected_memberships = vec![jon_doe(), jonette_snow()];
                replace_memberships(&mut connection, &expected_memberships).unwrap();
                let state = MembershipsState::load_memberships(&mut connection).unwrap();
                assert_eq!(
                    MembershipsState::new(
                        Some(FileDetails::new(Utc::now().date_naive(), OsString::new())),
                        IndexedMemberships::from(expected_memberships)
                    ),
                    state
                );
            });
        }
    }
}
