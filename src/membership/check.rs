use crate::database::dao;
use crate::database::error::DatabaseError::R2d2;
use crate::error::{ApplicationError, Result};
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dto::checked_member::CheckResult::{Match, NoMatch, PartialMatch};
use dto::checked_member::{CheckResult, CheckedMember};
use dto::member_to_check::MemberToCheck;

/// For each member, look into the database to check whether there is a match.
/// Matches are made in the following order:
/// 1. If membership number and names match, that's a perfect match ✔
/// 2. If membership number and identity match, that's also a perfect match ✔
/// 3. If membership number matches, that's a partial match ✔
/// 4. If the member to check has a membership number, but this number doesn't match anything, that's a no match ✖
/// 5. If the names match, that's a partial match ✔
/// 6. If the identity matches, that's a partial match ✔
/// 7. If there has been no match so far, then that's a no match ✖
pub fn check_members<T: MemberToCheck>(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
    members_to_check: Vec<T>,
) -> Result<Vec<CheckedMember<T>>> {
    Ok({
        let mut result = vec![];

        for member_to_check in members_to_check.into_iter() {
            let mut connection = pool
                .get()
                .map_err(|error| ApplicationError::Database(R2d2(error.to_string())))?;
            let checked_member = CheckedMember::new(
                member_to_check.clone(),
                check_member(&mut connection, &member_to_check)?,
            );
            result.push(checked_member);
        }

        result
    })
}

/// Look into the database to check whether there is a match.
/// Matches are made in the following order:
/// 1. If membership number and names match, that's a perfect match ✔
/// 2. If membership number and identity match, that's also a perfect match ✔
/// 3. If membership number matches, that's a partial match ✔
/// 4. If the member to check has a membership number, but this number doesn't match anything, that's a no match ✖
/// 5. If the names match, that's a partial match ✔
/// 6. If the identity matches, that's a partial match ✔
/// 7. If there has been no match so far, then that's a no match ✖
fn check_member<T: MemberToCheck>(
    connection: &mut SqliteConnection,
    member_to_check: &T,
) -> Result<CheckResult> {
    let membership_number = member_to_check.membership_num();
    let first_name = member_to_check.first_name();
    let last_name = member_to_check.last_name();
    let identity = member_to_check.identity();

    if membership_number.is_some() {
        if first_name.is_some() && last_name.is_some() {
            let membership_number = membership_number
                .clone()
                .expect("There should be a value at this point");
            let last_name = last_name
                .clone()
                .expect("There should be a value at this point");
            let first_name = first_name
                .clone()
                .expect("There should be a value at this point");
            if let Some(membership) = dao::membership::find::by_num_last_name_first_name(
                connection,
                &membership_number,
                &last_name,
                &first_name,
            )? {
                return Ok(Match(membership));
            }
        }

        if identity.is_some() {
            let membership_number = membership_number
                .clone()
                .expect("There should be a value at this point");
            let identity = identity
                .clone()
                .expect("There should be a value at this point");
            if let Some(membership) =
                dao::membership::find::by_num_identity(connection, &membership_number, &identity)?
            {
                return Ok(Match(membership));
            }
        }

        let membership_number = membership_number
            .clone()
            .expect("There should be a value at this point");
        if let Some(membership) = dao::membership::find::by_num(connection, &membership_number)? {
            return Ok(PartialMatch(membership));
        }

        // In case the membership number is provided, but it doesn't match anything,
        // then we consider there is no match, even though names or identity could match.
        return Ok(NoMatch);
    }

    if first_name.is_some() && last_name.is_some() {
        let last_name = last_name
            .clone()
            .expect("There should be a value at this point");
        let first_name = first_name
            .clone()
            .expect("There should be a value at this point");
        if let Some(membership) =
            dao::membership::find::by_last_name_first_name(connection, &last_name, &first_name)?
        {
            return Ok(PartialMatch(membership));
        }
    }

    if identity.is_some() {
        let identity = identity
            .clone()
            .expect("There should be a value at this point");
        if let Some(membership) = dao::membership::find::by_identity(connection, &identity)? {
            return Ok(PartialMatch(membership));
        }
    }

    Ok(NoMatch)
}

#[cfg(test)]
mod tests {
    mod check_members {
        use crate::database::dao::membership::replace_memberships;
        use crate::database::with_temp_database;
        use crate::membership::check::check_members;
        use dto::checked_member::CheckResult::{Match, NoMatch};
        use dto::checked_member::CheckedMember;
        use dto::csv_member::CsvMember;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    Some(MEMBERSHIP_NUMBER.to_owned()),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    vec![CheckedMember::new(
                        member_to_check.clone(),
                        Match(membership)
                    )],
                    check_members(&pool, vec![member_to_check]).unwrap()
                );
            });
        }

        #[test]
        fn fail() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
                let member_to_check = CsvMember::new(
                    Some(invalid_membership_number),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    vec![CheckedMember::new(member_to_check.clone(), NoMatch)],
                    check_members(&pool, vec![member_to_check]).unwrap()
                );
            });
        }
    }

    mod check_member {
        use crate::database::dao::membership::replace_memberships;
        use crate::database::with_temp_database;
        use crate::membership::check::check_member;
        use chrono::Months;
        use dto::checked_member::CheckResult::{Match, NoMatch, PartialMatch};
        use dto::csv_member::CsvMember;
        use dto::membership::Membership;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    Some(MEMBERSHIP_NUMBER.to_owned()),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    Match(membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn success_when_membership_number_prepended_with_0() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    Some(format!("0{MEMBERSHIP_NUMBER}")), // Prepending with a 0 should not change anything
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    Match(membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn match_when_membership_num_last_name_first_name_not_trimmed() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    Some(format!("  {MEMBERSHIP_NUMBER} ")),
                    None,
                    Some(format!(" {MEMBER_NAME}  ")),
                    Some(format!("{MEMBER_FIRST_NAME}  ")),
                );

                assert_eq!(
                    Match(membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn match_when_num_and_identity() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    Some(MEMBERSHIP_NUMBER.to_owned()), // Prepending with a 0 should not change anything
                    Some(format!("{} {}", MEMBER_NAME, MEMBER_FIRST_NAME)),
                    None,
                    None,
                );

                assert_eq!(
                    Match(membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn partial_match_when_identity() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let member_to_check = CsvMember::new(
                    None,
                    Some(format!("{MEMBER_NAME} {MEMBER_FIRST_NAME}")),
                    None,
                    None,
                );

                assert_eq!(
                    PartialMatch(membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn fail() {
            with_temp_database(|pool| {
                let membership = get_expected_membership();
                let mut connection = pool.get().unwrap();
                replace_memberships(&mut connection, &[membership.clone()]).unwrap();
                let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
                let member_to_check = CsvMember::new(
                    Some(invalid_membership_number),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    NoMatch,
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn get_better_match() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let matching_membership = get_expected_membership();
                let partial_matching_membership = Membership::new(
                    "Not the right name".to_owned(),
                    "Not the right first name either".to_owned(),
                    matching_membership.gender().to_owned(),
                    matching_membership.birthdate().to_owned(),
                    matching_membership.age().to_owned(),
                    matching_membership.membership_number().to_owned(),
                    matching_membership.email_address().to_owned(),
                    matching_membership.payed().to_owned(),
                    matching_membership.end_date().to_owned(),
                    matching_membership.expired().to_owned(),
                    matching_membership.club().to_owned(),
                    matching_membership.structure_code().to_owned(),
                );
                let not_matching_membership = Membership::new(
                    "Not the right name".to_owned(),
                    "Not the right first name either".to_owned(),
                    matching_membership.gender().to_owned(),
                    matching_membership.birthdate().to_owned(),
                    matching_membership.age().to_owned(),
                    "Also wrong membership number".to_owned(),
                    matching_membership.email_address().to_owned(),
                    matching_membership.payed().to_owned(),
                    matching_membership.end_date().to_owned(),
                    matching_membership.expired().to_owned(),
                    matching_membership.club().to_owned(),
                    matching_membership.structure_code().to_owned(),
                );
                replace_memberships(
                    &mut connection,
                    &[
                        matching_membership.clone(),
                        partial_matching_membership,
                        not_matching_membership,
                    ],
                )
                .unwrap();

                let member_to_check = CsvMember::new(
                    Some(MEMBERSHIP_NUMBER.to_owned()),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    Match(matching_membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }

        #[test]
        fn get_better_match_when_different_end_dates() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let newest_membership = get_expected_membership();
                let oldest_membership = Membership::new(
                    newest_membership.name().to_owned(),
                    newest_membership.first_name().to_owned(),
                    newest_membership.gender().to_owned(),
                    newest_membership.birthdate().to_owned(),
                    newest_membership.age().to_owned(),
                    newest_membership.membership_number().to_owned(),
                    newest_membership.email_address().to_owned(),
                    newest_membership.payed().to_owned(),
                    newest_membership
                        .end_date()
                        .to_owned()
                        .checked_sub_months(Months::new(12))
                        .unwrap(),
                    newest_membership.expired().to_owned(),
                    newest_membership.club().to_owned(),
                    newest_membership.structure_code().to_owned(),
                );
                replace_memberships(
                    &mut connection,
                    &[newest_membership.clone(), oldest_membership],
                )
                .unwrap();
                let member_to_check = CsvMember::new(
                    Some(MEMBERSHIP_NUMBER.to_owned()),
                    None,
                    Some(MEMBER_NAME.to_owned()),
                    Some(MEMBER_FIRST_NAME.to_owned()),
                );

                assert_eq!(
                    Match(newest_membership),
                    check_member(&mut connection, &member_to_check).unwrap()
                );
            });
        }
    }
}
