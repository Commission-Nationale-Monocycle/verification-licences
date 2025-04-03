use super::Result;
use crate::database::dao::last_update::UpdatableElement;
use crate::database::model::membership::Membership;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

#[allow(dead_code)]
pub fn retrieve_memberships(
    connection: &mut SqliteConnection,
) -> Result<Vec<dto::membership::Membership>> {
    let results = crate::database::schema::membership::dsl::membership
        .select(Membership::as_select())
        .load(connection)?;

    let memberships = {
        let mut memberships = Vec::new();
        for result in results {
            memberships.push(dto::membership::Membership::try_from(result)?);
        }

        memberships
    };

    Ok(memberships)
}

fn delete_all(connection: &mut SqliteConnection) -> Result<usize> {
    let count = diesel::delete(crate::database::schema::membership::table).execute(connection)?;

    Ok(count)
}

fn insert_all(
    connection: &mut SqliteConnection,
    memberships: &[dto::membership::Membership],
) -> Result<usize> {
    use crate::database::schema::membership::*;

    let memberships = memberships
        .iter()
        .map(|membership| {
            (
                last_name.eq(membership.name().clone()),
                first_name.eq(membership.first_name().clone()),
                gender.eq(membership.gender().clone()),
                birthdate.eq(membership.birthdate().map(|b| b.to_string())),
                age.eq(membership.age().map(|a| a as i32)),
                membership_number.eq(membership.membership_number().clone()),
                email_address.eq(membership.email_address().clone()),
                payed.eq(*membership.payed()),
                end_date.eq(membership.end_date().to_string()),
                expired.eq(*membership.expired()),
                club.eq(membership.club().clone()),
                structure_code.eq(membership.structure_code().clone()),
            )
        })
        .collect::<Vec<_>>();
    // Limit of 32766 parameters in a query for SQLite > 3.32.0.
    // As each line has 12 parameters, we have a theoretic maximum of 32 766 / 12 = 2730,5.
    // Let's say we insert 2500 elements at a time.
    let memberships = memberships.chunks(2500);

    let mut count = 0;
    for chunk in memberships {
        count += diesel::insert_into(crate::database::schema::membership::table)
            .values(chunk)
            .execute(connection)?;
    }

    super::last_update::update(connection, &UpdatableElement::Memberships)?;

    Ok(count)
}

/// Delete all known memberships and replace them with new ones.
/// Return the number of deleted memberships and the number of inserted memberships.
#[allow(dead_code)]
pub fn replace_memberships(
    connection: &mut SqliteConnection,
    memberships: &[dto::membership::Membership],
) -> Result<(usize, usize)> {
    let deleted_count = delete_all(connection)?;
    let inserted_count = insert_all(connection, memberships)?;

    Ok((deleted_count, inserted_count))
}

#[cfg(test)]
mod tests {
    use crate::database::schema::membership::*;
    use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
    use diesel::prelude::*;

    fn establish_connection() -> SqliteConnection {
        crate::database::establish_connection().unwrap()
    }

    fn populate_db(connection: &mut SqliteConnection) -> Vec<dto::membership::Membership> {
        let expected_memberships = vec![jon_doe(), jonette_snow()];
        let memberships = expected_memberships
            .clone()
            .into_iter()
            .map(|membership| {
                (
                    last_name.eq(membership.name().clone()),
                    first_name.eq(membership.first_name().clone()),
                    gender.eq(membership.gender().clone()),
                    birthdate.eq(membership.birthdate().map(|b| b.to_string())),
                    age.eq(membership.age().map(|a| a as i32)),
                    membership_number.eq(membership.membership_number().clone()),
                    email_address.eq(membership.email_address().clone()),
                    payed.eq(*membership.payed()),
                    end_date.eq(membership.end_date().to_string()),
                    expired.eq(*membership.expired()),
                    club.eq(membership.club().clone()),
                    structure_code.eq(membership.structure_code().clone()),
                )
            })
            .collect::<Vec<_>>();

        diesel::insert_into(crate::database::schema::membership::table)
            .values(&memberships)
            .execute(connection)
            .unwrap();

        expected_memberships
    }

    mod retrieve_memberships {
        use crate::database::dao::membership::retrieve_memberships;
        use crate::database::dao::membership::tests::{establish_connection, populate_db};
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|| {
                let mut connection = establish_connection();
                let expected_memberships = populate_db(&mut connection);

                let result = retrieve_memberships(&mut connection).unwrap();
                assert_eq!(expected_memberships, result);
            })
        }
    }

    mod delete_all {
        use crate::database::dao::membership::delete_all;
        use crate::database::dao::membership::tests::{establish_connection, populate_db};
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|| {
                let mut connection = establish_connection();
                let expected_memberships = populate_db(&mut connection);

                let result = delete_all(&mut connection).unwrap();
                assert_eq!(expected_memberships.len(), result);
            })
        }

        #[test]
        fn success_when_already_empty() {
            with_temp_database(|| {
                let mut connection = establish_connection();

                let result = delete_all(&mut connection).unwrap();
                assert_eq!(0, result);
            })
        }
    }

    mod insert_all {
        use crate::database::dao::last_update::{UpdatableElement, get_last_update};
        use crate::database::dao::membership::insert_all;
        use crate::database::dao::membership::tests::establish_connection;
        use crate::database::model::membership::Membership;
        use crate::database::with_temp_database;
        use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
        use chrono::Utc;
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        fn test_insert(expected_memberships: &[dto::membership::Membership]) {
            with_temp_database(|| {
                let mut connection = establish_connection();

                let result = insert_all(&mut connection, expected_memberships).unwrap();
                assert_eq!(expected_memberships.len(), result);

                let results = crate::database::schema::membership::dsl::membership
                    .select(Membership::as_select())
                    .load(&mut connection)
                    .unwrap();

                let memberships = {
                    let mut memberships = Vec::new();
                    for result in results {
                        memberships.push(dto::membership::Membership::try_from(result).unwrap());
                    }

                    memberships
                };

                assert_eq!(expected_memberships, memberships);
                get_last_update(&mut connection, &UpdatableElement::Memberships)
                    .unwrap()
                    .unwrap(); // The last_update table should have been updated
            })
        }

        #[test]
        fn success() {
            let expected_memberships = vec![jon_doe(), jonette_snow()];
            test_insert(&expected_memberships);
        }

        /// A long list of memberships to insert could make the query fail if it isn't correctly chunked.
        #[test]
        fn success_with_long_list() {
            let expected_memberships = (0..10000)
                .map(|i| {
                    dto::membership::Membership::new(
                        i.to_string(),
                        i.to_string(),
                        i.to_string(),
                        None,
                        None,
                        i.to_string(),
                        i.to_string(),
                        true,
                        Utc::now().date_naive(),
                        false,
                        i.to_string(),
                        i.to_string(),
                    )
                })
                .collect::<Vec<_>>();
            test_insert(&expected_memberships);
        }
    }

    mod replace_memberships {
        use crate::database::dao::last_update::{UpdatableElement, get_last_update};
        use crate::database::dao::membership::replace_memberships;
        use crate::database::dao::membership::tests::establish_connection;
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::model::membership::Membership;
        use crate::database::with_temp_database;
        use crate::membership::indexed_memberships::tests::{
            jon_doe_previous_membership, other_jon_doe,
        };
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        #[test]
        fn success() {
            with_temp_database(|| {
                let mut connection = establish_connection();
                let initial_memberships = populate_db(&mut connection);
                let expected_memberships = vec![jon_doe_previous_membership(), other_jon_doe()];

                let result = replace_memberships(&mut connection, &expected_memberships).unwrap();
                assert_eq!(
                    (initial_memberships.len(), expected_memberships.len()),
                    result
                );

                let results = crate::database::schema::membership::dsl::membership
                    .select(Membership::as_select())
                    .load(&mut connection)
                    .unwrap();

                let memberships = {
                    let mut memberships = Vec::new();
                    for result in results {
                        memberships.push(dto::membership::Membership::try_from(result).unwrap());
                    }

                    memberships
                };

                assert_eq!(expected_memberships, memberships);
                get_last_update(&mut connection, &UpdatableElement::Memberships)
                    .unwrap()
                    .unwrap(); // The last_update table should have been updated
            })
        }
    }
}
