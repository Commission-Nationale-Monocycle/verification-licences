use crate::error::Result;
use diesel::SqliteConnection;
use dto::member_to_look_up::MemberToLookUp;
use dto::membership::Membership;
use std::collections::BTreeSet;

/// Load all memberships filtered by given information.
/// If no information is given, then nothing is retrieved.
pub fn look_member_up(
    connection: &mut SqliteConnection,
    member_to_look_up: &MemberToLookUp,
) -> Result<BTreeSet<Membership>> {
    if member_to_look_up.membership_num().is_none()
        && member_to_look_up.last_name().is_none()
        && member_to_look_up.first_name().is_none()
    {
        return Ok(BTreeSet::new());
    }

    Ok(
        crate::database::dao::membership::find::all::by_member_to_lookup(
            connection,
            member_to_look_up,
        )?,
    )
}

#[cfg(test)]
mod tests {
    mod look_member_up {
        use crate::database::dao::membership::replace_memberships;
        use crate::database::with_temp_database;
        use crate::membership::look_up::look_member_up;
        use crate::membership::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use dto::member_to_look_up::MemberToLookUp;
        use dto::membership::Membership;
        use std::collections::BTreeSet;

        #[test]
        fn by_membership_num() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                replace_memberships(
                    &mut connection,
                    &[
                        jonette_snow(),
                        jon_doe(),
                        jon_doe_previous_membership(),
                        other_jon_doe(),
                    ],
                )
                .unwrap();
                let member_to_look_up =
                    MemberToLookUp::new(Some(jon_doe().membership_number().to_owned()), None, None);

                let result = look_member_up(&mut connection, &member_to_look_up).unwrap();

                assert_eq!(
                    BTreeSet::from([jon_doe_previous_membership(), jon_doe()]),
                    result
                );
            });
        }

        #[test]
        fn by_last_name() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                replace_memberships(
                    &mut connection,
                    &[
                        jonette_snow(),
                        jon_doe(),
                        jon_doe_previous_membership(),
                        other_jon_doe(),
                    ],
                )
                .unwrap();
                let member_to_look_up =
                    MemberToLookUp::new(None, Some(jon_doe().name().to_owned()), None);

                let result = look_member_up(&mut connection, &member_to_look_up).unwrap();

                assert_eq!(
                    BTreeSet::from([jon_doe_previous_membership(), jon_doe(), other_jon_doe()]),
                    result
                );
            })
        }

        #[test]
        fn by_first_name() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                replace_memberships(
                    &mut connection,
                    &[
                        jonette_snow(),
                        jon_doe(),
                        jon_doe_previous_membership(),
                        other_jon_doe(),
                    ],
                )
                .unwrap();
                let member_to_look_up =
                    MemberToLookUp::new(None, None, Some(jon_doe().first_name().to_owned()));

                let result = look_member_up(&mut connection, &member_to_look_up).unwrap();

                assert_eq!(
                    BTreeSet::from([jon_doe(), other_jon_doe(), jon_doe_previous_membership()]),
                    result
                );
            });
        }

        #[test]
        fn no_criteria() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                replace_memberships(
                    &mut connection,
                    &[
                        jonette_snow(),
                        jon_doe(),
                        jon_doe_previous_membership(),
                        other_jon_doe(),
                    ],
                )
                .unwrap();
                let member_to_look_up = MemberToLookUp::new(None, None, None);

                let result = look_member_up(&mut connection, &member_to_look_up).unwrap();

                assert_eq!(BTreeSet::<Membership>::new(), result);
            });
        }
    }
}
