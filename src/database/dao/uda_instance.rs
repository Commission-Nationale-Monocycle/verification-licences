use super::Result;
use crate::database::dao::last_update::UpdatableElement;
use crate::database::model::uda_instance::UdaInstance;
use crate::database::schema::uda_instance::{name, slug, url};
use diesel::prelude::*;

pub fn retrieve_all(connection: &mut SqliteConnection) -> Result<Vec<dto::uda_instance::Instance>> {
    let results = crate::database::schema::uda_instance::dsl::uda_instance
        .select(UdaInstance::as_select())
        .load(connection)?;

    Ok(results
        .into_iter()
        .map(dto::uda_instance::Instance::from)
        .collect())
}

fn delete_all(connection: &mut SqliteConnection) -> Result<usize> {
    let count = diesel::delete(crate::database::schema::uda_instance::table).execute(connection)?;

    Ok(count)
}

fn insert_all(
    connection: &mut SqliteConnection,
    uda_instances: &[dto::uda_instance::Instance],
) -> Result<usize> {
    let uda_instances = uda_instances
        .iter()
        .map(|instance| {
            (
                slug.eq(instance.slug()),
                name.eq(instance.name()),
                url.eq(instance.url()),
            )
        })
        .collect::<Vec<_>>();

    let count = diesel::insert_into(crate::database::schema::uda_instance::table)
        .values(uda_instances)
        .execute(connection)?;

    super::last_update::update(connection, &UpdatableElement::UdaInstances)?;

    Ok(count)
}

pub fn replace_all(
    connection: &mut SqliteConnection,
    uda_instances: &[dto::uda_instance::Instance],
) -> Result<(usize, usize)> {
    let deleted_count = delete_all(connection)?;
    let inserted_count = insert_all(connection, uda_instances)?;

    Ok((deleted_count, inserted_count))
}

#[cfg(test)]
mod tests {
    use crate::database::schema::uda_instance::{name, slug, url};
    use diesel::prelude::*;
    use dto::uda_instance::Instance;

    fn get_test_instances() -> Vec<Instance> {
        vec![
            Instance::new(
                "cfm2025".to_owned(),
                "CFM 2025".to_owned(),
                "https://cfm2025.reg.unicycling-software.com/".to_owned(),
            ),
            Instance::new(
                "cfm2024".to_owned(),
                "CFM 2024".to_owned(),
                "https://cfm2024.reg.unicycling-software.com/".to_owned(),
            ),
            Instance::new(
                "unicon2024".to_owned(),
                "CFM 2024".to_owned(),
                "https://unicon2024.reg.unicycling-software.com/".to_owned(),
            ),
        ]
    }

    fn populate_db(connection: &mut SqliteConnection) -> Vec<Instance> {
        let expected_instances = get_test_instances();
        let instances = expected_instances
            .clone()
            .into_iter()
            .map(|instance| {
                (
                    slug.eq(instance.slug().clone()),
                    name.eq(instance.name().clone()),
                    url.eq(instance.url().clone()),
                )
            })
            .collect::<Vec<_>>();

        diesel::insert_into(crate::database::schema::uda_instance::table)
            .values(&instances)
            .execute(connection)
            .unwrap();

        expected_instances
    }

    mod retrieve_all {
        use crate::database::dao::uda_instance::retrieve_all;
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let expected_instances = super::populate_db(&mut connection);

                let result = retrieve_all(&mut connection).unwrap();
                assert_eq!(expected_instances, result);
            })
        }
    }

    mod delete_all {
        use crate::database::dao::uda_instance::delete_all;
        use crate::database::with_temp_database;

        #[test]
        fn success() {
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();
                let expected_instances = super::populate_db(&mut connection);

                let result = delete_all(&mut connection).unwrap();
                assert_eq!(expected_instances.len(), result);
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
        use crate::database::dao::uda_instance::insert_all;
        use crate::database::dao::uda_instance::tests::get_test_instances;
        use crate::database::model::uda_instance::UdaInstance;
        use crate::database::with_temp_database;
        use diesel::prelude::*;

        #[test]
        fn success() {
            let expected_instances = get_test_instances();
            with_temp_database(|pool| {
                let mut connection = pool.get().unwrap();

                let result = insert_all(&mut connection, &expected_instances).unwrap();
                assert_eq!(expected_instances.len(), result);

                let results = crate::database::schema::uda_instance::dsl::uda_instance
                    .select(UdaInstance::as_select())
                    .load(&mut connection)
                    .unwrap();

                let uda_instances = results
                    .into_iter()
                    .map(dto::uda_instance::Instance::from)
                    .collect::<Vec<_>>();

                assert_eq!(expected_instances, uda_instances);
                get_last_update(&mut connection, &UpdatableElement::UdaInstances)
                    .unwrap()
                    .unwrap(); // The last_update table should have been updated
            })
        }

        mod replace_all {
            use crate::database::dao::last_update::{UpdatableElement, get_last_update};
            use crate::database::dao::uda_instance::replace_all;
            use crate::database::dao::uda_instance::tests::{get_test_instances, populate_db};
            use crate::database::model::uda_instance::UdaInstance;
            use crate::database::with_temp_database;
            use diesel::prelude::*;

            #[test]
            fn success() {
                with_temp_database(|pool| {
                    let mut connection = pool.get().unwrap();
                    let initial_instances = populate_db(&mut connection);
                    let expected_instances = get_test_instances();

                    let result = replace_all(&mut connection, &expected_instances).unwrap();
                    assert_eq!((initial_instances.len(), expected_instances.len()), result);

                    let results = crate::database::schema::uda_instance::dsl::uda_instance
                        .select(UdaInstance::as_select())
                        .load(&mut connection)
                        .unwrap();

                    let uda_instances = results
                        .into_iter()
                        .map(dto::uda_instance::Instance::from)
                        .collect::<Vec<_>>();

                    assert_eq!(expected_instances, uda_instances);
                    get_last_update(&mut connection, &UpdatableElement::UdaInstances)
                        .unwrap()
                        .unwrap(); // The last_update table should have been updated
                })
            }
        }
    }
}
