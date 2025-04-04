use super::Result;
use crate::database::dao::last_update::UpdatableElement;
use crate::database::model::membership::Membership;
use crate::tools::normalize;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

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
                normalized_membership_number.eq(normalize(membership.membership_number())),
                normalized_last_name.eq(normalize(membership.name())),
                normalized_first_name.eq(normalize(membership.first_name())),
                normalized_last_name_first_name.eq(format!(
                    "{}{}",
                    normalize(membership.name()),
                    normalize(membership.first_name()),
                )),
                normalized_first_name_last_name.eq(format!(
                    "{}{}",
                    normalize(membership.first_name()),
                    normalize(membership.name()),
                )),
            )
        })
        .collect::<Vec<_>>();
    // Limit of 32766 parameters in a query for SQLite > 3.32.0.
    // As each line has 17 parameters, we have a theoretic maximum of 32 766 / 17 = 1927,4.
    let memberships = memberships.chunks(1927);

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
pub fn replace_memberships(
    connection: &mut SqliteConnection,
    memberships: &[dto::membership::Membership],
) -> Result<(usize, usize)> {
    let deleted_count = delete_all(connection)?;
    let inserted_count = insert_all(connection, memberships)?;

    Ok((deleted_count, inserted_count))
}

#[allow(dead_code)]
pub(crate) mod find {
    use super::super::Result;
    use crate::database::model::membership::Membership;
    use crate::database::schema::membership::{
        end_date, normalized_first_name, normalized_first_name_last_name, normalized_last_name,
        normalized_last_name_first_name, normalized_membership_number,
    };
    use crate::tools::normalize;
    use diesel::dsl::{Asc, Desc};
    use diesel::{
        BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
        SqliteConnection,
    };

    fn by_num(
        connection: &mut SqliteConnection,
        membership_number: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Membership>> {
        let results = crate::database::schema::membership::dsl::membership
            .filter(normalized_membership_number.eq(normalize(membership_number)))
            .order(get_order())
            .limit(limit.unwrap_or(i64::MAX))
            .select(Membership::as_select())
            .load(connection)?;

        Ok(results)
    }

    fn by_num_identity(
        connection: &mut SqliteConnection,
        membership_number: &str,
        identity: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Membership>> {
        let normalized_identity = normalize(identity);
        let results = crate::database::schema::membership::dsl::membership
            .filter(normalized_membership_number.eq(normalize(membership_number)))
            .filter(
                normalized_last_name_first_name
                    .eq(&normalized_identity)
                    .or(normalized_first_name_last_name.eq(&normalized_identity)),
            )
            .order(get_order())
            .limit(limit.unwrap_or(i64::MAX))
            .select(Membership::as_select())
            .load(connection)?;

        Ok(results)
    }

    fn by_num_last_name_first_name(
        connection: &mut SqliteConnection,
        membership_number: &str,
        last_name: &str,
        first_name: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Membership>> {
        let results = crate::database::schema::membership::dsl::membership
            .filter(normalized_membership_number.eq(normalize(membership_number)))
            .filter(normalized_last_name.eq(normalize(last_name)))
            .filter(normalized_first_name.eq(normalize(first_name)))
            .order(get_order())
            .limit(limit.unwrap_or(i64::MAX))
            .select(Membership::as_select())
            .load(connection)?;

        Ok(results)
    }

    fn by_identity(
        connection: &mut SqliteConnection,
        identity: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Membership>> {
        let normalized_identity = normalize(identity);
        let results = crate::database::schema::membership::dsl::membership
            .filter(
                normalized_last_name_first_name
                    .eq(&normalized_identity)
                    .or(normalized_first_name_last_name.eq(&normalized_identity)),
            )
            .order(get_order())
            .limit(limit.unwrap_or(i64::MAX))
            .select(Membership::as_select())
            .load(connection)?;

        Ok(results)
    }

    fn by_last_name_first_name(
        connection: &mut SqliteConnection,
        last_name: &str,
        first_name: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Membership>> {
        let results = crate::database::schema::membership::dsl::membership
            .filter(normalized_last_name.eq(normalize(last_name)))
            .filter(normalized_first_name.eq(normalize(first_name)))
            .order(get_order())
            .limit(limit.unwrap_or(i64::MAX))
            .select(Membership::as_select())
            .load(connection)?;

        Ok(results)
    }

    fn get_order() -> (
        Desc<end_date>,
        Asc<normalized_membership_number>,
        Asc<normalized_last_name_first_name>,
    ) {
        (
            end_date.desc(),
            normalized_membership_number.asc(),
            normalized_last_name_first_name.asc(),
        )
    }

    pub(crate) mod all {
        use super::super::Result;
        use crate::database::model::membership::Membership;
        use diesel::SqliteConnection;

        pub fn by_num(
            connection: &mut SqliteConnection,
            membership_number: &str,
        ) -> Result<Vec<dto::membership::Membership>> {
            let results = super::by_num(connection, membership_number, None)?;
            convert_to_dto(results)
        }

        pub fn by_num_identity(
            connection: &mut SqliteConnection,
            membership_number: &str,
            identity: &str,
        ) -> Result<Vec<dto::membership::Membership>> {
            let results = super::by_num_identity(connection, membership_number, identity, None)?;
            convert_to_dto(results)
        }

        pub fn by_num_last_name_first_name(
            connection: &mut SqliteConnection,
            membership_number: &str,
            last_name: &str,
            first_name: &str,
        ) -> Result<Vec<dto::membership::Membership>> {
            let results = super::by_num_last_name_first_name(
                connection,
                membership_number,
                last_name,
                first_name,
                None,
            )?;
            convert_to_dto(results)
        }

        pub fn by_identity(
            connection: &mut SqliteConnection,
            identity: &str,
        ) -> Result<Vec<dto::membership::Membership>> {
            let results = super::by_identity(connection, identity, None)?;
            convert_to_dto(results)
        }

        pub fn by_last_name_first_name(
            connection: &mut SqliteConnection,
            last_name: &str,
            first_name: &str,
        ) -> Result<Vec<dto::membership::Membership>> {
            let results = super::by_last_name_first_name(connection, last_name, first_name, None)?;

            convert_to_dto(results)
        }

        fn convert_to_dto(results: Vec<Membership>) -> Result<Vec<dto::membership::Membership>> {
            Ok({
                let mut memberships = vec![];

                for membership in results {
                    memberships.push(dto::membership::Membership::try_from(membership)?);
                }

                memberships
            })
        }
    }

    pub(crate) mod first {
        use super::super::Result;
        use crate::database::model::membership::Membership;
        use diesel::SqliteConnection;

        pub fn by_num(
            connection: &mut SqliteConnection,
            membership_number: &str,
        ) -> Result<Option<dto::membership::Membership>> {
            let results = super::by_num(connection, membership_number, Some(1))?;
            convert_to_dto(results)
        }

        pub fn by_num_identity(
            connection: &mut SqliteConnection,
            membership_number: &str,
            identity: &str,
        ) -> Result<Option<dto::membership::Membership>> {
            let results = super::by_num_identity(connection, membership_number, identity, Some(1))?;
            convert_to_dto(results)
        }

        pub fn by_num_last_name_first_name(
            connection: &mut SqliteConnection,
            membership_number: &str,
            last_name: &str,
            first_name: &str,
        ) -> Result<Option<dto::membership::Membership>> {
            let results = super::by_num_last_name_first_name(
                connection,
                membership_number,
                last_name,
                first_name,
                Some(1),
            )?;
            convert_to_dto(results)
        }

        pub fn by_identity(
            connection: &mut SqliteConnection,
            identity: &str,
        ) -> Result<Option<dto::membership::Membership>> {
            let results = super::by_identity(connection, identity, Some(1))?;
            convert_to_dto(results)
        }

        pub fn by_last_name_first_name(
            connection: &mut SqliteConnection,
            last_name: &str,
            first_name: &str,
        ) -> Result<Option<dto::membership::Membership>> {
            let results =
                super::by_last_name_first_name(connection, last_name, first_name, Some(1))?;

            convert_to_dto(results)
        }

        fn convert_to_dto(results: Vec<Membership>) -> Result<Option<dto::membership::Membership>> {
            if let Some(membership) = results.first().cloned() {
                Ok(Some(dto::membership::Membership::try_from(membership)?))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::database::schema::membership::*;
    use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
    use crate::tools::normalize;
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
                    normalized_membership_number.eq(normalize(membership.membership_number())),
                    normalized_last_name.eq(normalize(membership.name())),
                    normalized_first_name.eq(normalize(membership.first_name())),
                    normalized_last_name_first_name.eq(format!(
                        "{}{}",
                        normalize(membership.name()),
                        normalize(membership.first_name()),
                    )),
                    normalized_first_name_last_name.eq(format!(
                        "{}{}",
                        normalize(membership.first_name()),
                        normalize(membership.name()),
                    )),
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
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let expected_memberships = populate_db(&mut connection);

                let result = retrieve_memberships(&mut connection).unwrap();
                assert_eq!(expected_memberships, result);
            })
        }
    }

    mod delete_all {
        use crate::database::dao::membership::delete_all;
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let expected_memberships = populate_db(&mut connection);

                let result = delete_all(&mut connection).unwrap();
                assert_eq!(expected_memberships.len(), result);
            })
        }

        #[test]
        fn success_when_already_empty() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();

                let result = delete_all(&mut connection).unwrap();
                assert_eq!(0, result);
            })
        }
    }

    mod insert_all {
        use crate::database::dao::last_update::{UpdatableElement, get_last_update};
        use crate::database::dao::membership::insert_all;
        use crate::database::model::membership::Membership;
        use crate::database::with_temp_database;
        use crate::membership::indexed_memberships::tests::{jon_doe, jonette_snow};
        use chrono::Utc;
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        fn test_insert(expected_memberships: &[dto::membership::Membership]) {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();

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
        use crate::database::dao::membership::tests::populate_db;
        use crate::database::model::membership::Membership;
        use crate::database::with_temp_database;
        use crate::membership::indexed_memberships::tests::{
            jon_doe_previous_membership, other_jon_doe,
        };
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
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

    mod find {
        mod all {
            mod by_num {
                use crate::database::dao::membership::find::all::by_num;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};
                use dto::membership::Membership;

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num(&mut connection, &num).unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_both() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(
                            &mut connection,
                            &[membership.clone(), old_membership.clone()],
                        )
                        .unwrap();

                        let result = by_num(&mut connection, &num).unwrap();
                        assert_eq!(vec![membership.clone(), old_membership.clone()], result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num(&mut connection, &num).unwrap();
                        assert_eq!(Vec::<Membership>::new(), result);
                    });
                }
            }

            mod by_num_identity {
                use crate::database::dao::membership::find::all::by_num_identity;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};
                use dto::membership::Membership;

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity).unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_the_only_one_by_reversed_identity() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &last_name, &first_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity).unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_all() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            first_name.clone(),
                            last_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            first_name.clone(),
                            last_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(
                            &mut connection,
                            &[membership.clone(), old_membership.clone()],
                        )
                        .unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity).unwrap();
                        assert_eq!(vec![membership, old_membership], result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity).unwrap();
                        assert_eq!(Vec::<Membership>::new(), result);
                    });
                }
            }

            mod by_num_last_name_first_name {
                use crate::database::dao::membership::find::all::by_num_last_name_first_name;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};
                use dto::membership::Membership;

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_all() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(
                            &mut connection,
                            &[membership.clone(), old_membership.clone()],
                        )
                        .unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap();
                        assert_eq!(vec![membership, old_membership], result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap();
                        assert_eq!(Vec::<Membership>::new(), result);
                    });
                }
            }

            mod by_identity {
                use crate::database::dao::membership::find::all::by_identity;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};
                use dto::membership::Membership;

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_the_only_one_by_reversed_identity() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &last_name, &first_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_all() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(
                            &mut connection,
                            &[membership.clone(), old_membership.clone()],
                        )
                        .unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap();
                        assert_eq!(vec![membership, old_membership], result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap();
                        assert_eq!(Vec::<Membership>::new(), result);
                    });
                }
            }

            mod by_last_name_first_name {
                use crate::database::dao::membership::find::all::by_last_name_first_name;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};
                use dto::membership::Membership;

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap();
                        assert_eq!(vec![membership], result);
                    });
                }

                #[test]
                fn find_all() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(
                            &mut connection,
                            &[membership.clone(), old_membership.clone()],
                        )
                        .unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap();
                        assert_eq!(vec![membership, old_membership], result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap();
                        assert_eq!(Vec::<Membership>::new(), result);
                    });
                }
            }
        }

        mod first {
            mod by_num {
                use crate::database::dao::membership::find::first::by_num;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num(&mut connection, &num).unwrap().unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_last_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            "Doe".to_owned(),
                            "Jon".to_owned(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone(), old_membership]).unwrap();

                        let result = by_num(&mut connection, &num).unwrap().unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num(&mut connection, &num).unwrap();
                        assert_eq!(None, result);
                    });
                }
            }

            mod by_num_identity {
                use crate::database::dao::membership::find::first::by_num_identity;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity)
                            .unwrap()
                            .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_only_one_by_reversed_identity() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &last_name, &first_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity)
                            .unwrap()
                            .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_last_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            first_name.clone(),
                            last_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            first_name.clone(),
                            last_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone(), old_membership]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity)
                            .unwrap()
                            .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num_identity(&mut connection, &num, &identity).unwrap();
                        assert_eq!(None, result);
                    });
                }
            }

            mod by_num_last_name_first_name {
                use crate::database::dao::membership::find::first::by_num_last_name_first_name;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap()
                        .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_last_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone(), old_membership]).unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap()
                        .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_num_last_name_first_name(
                            &mut connection,
                            &num,
                            &last_name,
                            &first_name,
                        )
                        .unwrap();
                        assert_eq!(None, result);
                    });
                }
            }

            mod by_identity {
                use crate::database::dao::membership::find::first::by_identity;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap().unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_only_one_by_reversed_identity() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &last_name, &first_name);
                        let membership = dto::membership::Membership::new(
                            last_name,
                            first_name,
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap().unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_last_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone(), old_membership]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap().unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let identity = format!("{}{}", &first_name, &last_name);
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result = by_identity(&mut connection, &identity).unwrap();
                        assert_eq!(None, result);
                    });
                }
            }

            mod by_last_name_first_name {
                use crate::database::dao::membership::find::first::by_last_name_first_name;
                use crate::database::dao::membership::insert_all;
                use crate::database::with_temp_database;
                use chrono::{Months, Utc};

                #[test]
                fn find_the_only_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone()]).unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap()
                                .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn find_the_last_one() {
                    with_temp_database(|pool| {
                        let num = "123456".to_owned();
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num.clone(),
                            "address@test.com".to_owned(),
                            true,
                            Utc::now().date_naive(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let old_membership = dto::membership::Membership::new(
                            last_name.clone(),
                            first_name.clone(),
                            "M".to_owned(),
                            None,
                            None,
                            num,
                            "address@test.com".to_owned(),
                            true,
                            Utc::now()
                                .date_naive()
                                .checked_sub_months(Months::new(12))
                                .unwrap(),
                            false,
                            "club".to_owned(),
                            "A12345".to_owned(),
                        );

                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[membership.clone(), old_membership]).unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap()
                                .unwrap();
                        assert_eq!(membership, result);
                    });
                }

                #[test]
                fn none_matching() {
                    with_temp_database(|pool| {
                        let first_name = "Jon".to_owned();
                        let last_name = "Doe".to_owned();
                        let mut connection = pool.get().unwrap();

                        insert_all(&mut connection, &[]).unwrap();

                        let result =
                            by_last_name_first_name(&mut connection, &last_name, &first_name)
                                .unwrap();
                        assert_eq!(None, result);
                    });
                }
            }
        }
    }
}
