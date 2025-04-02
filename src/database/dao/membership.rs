use crate::database::error::DatabaseError;
use crate::database::model::membership::Membership;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

type Result<T, E = DatabaseError> = std::result::Result<T, E>;

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

    let count = diesel::insert_into(crate::database::schema::membership::table)
        .values(&memberships)
        .execute(connection)?;

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
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::tests::establish_connection;

        #[test]
        fn success() {
            let mut connection = establish_connection();
            let expected_memberships = populate_db(&mut connection);

            let result = retrieve_memberships(&mut connection).unwrap();
            assert_eq!(expected_memberships, result);
        }
    }

    mod delete_all {
        use crate::database::dao::membership::delete_all;
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::tests::establish_connection;

        #[test]
        fn success() {
            let mut connection = establish_connection();
            let expected_memberships = populate_db(&mut connection);

            let result = delete_all(&mut connection).unwrap();
            assert_eq!(expected_memberships.len(), result);
        }

        #[test]
        fn success_when_already_empty() {
            let mut connection = establish_connection();

            let result = delete_all(&mut connection).unwrap();
            assert_eq!(0, result);
        }
    }

    mod insert_all {
        use crate::database::dao::membership::insert_all;
        use crate::database::model::membership::Membership;
        use crate::database::tests::establish_connection;
        use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        #[test]
        fn success() {
            let mut connection = establish_connection();
            let expected_memberships = vec![jon_doe(), jonette_snow()];

            let result = insert_all(&mut connection, &expected_memberships).unwrap();
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
        }
    }

    mod replace_memberships {
        use crate::database::dao::membership::replace_memberships;
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::model::membership::Membership;
        use crate::database::tests::establish_connection;
        use crate::membership::indexed_memberships::tests::{
            jon_doe_previous_membership, other_jon_doe,
        };
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        #[test]
        fn success() {
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
        }
    }
}
