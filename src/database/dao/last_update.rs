use super::Result;
use crate::database::error::DatabaseError::CantUpdateLastUpdated;
use crate::database::model::last_update::LastUpdate;
use crate::database::schema::last_update::dsl::last_update;
use crate::database::schema::last_update::*;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum UpdatableElement {
    Memberships,
    UdaInstances,
}

impl Display for UpdatableElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_last_update(
    connection: &mut SqliteConnection,
    updatable_element: &UpdatableElement,
) -> Result<Option<NaiveDateTime>> {
    let result = last_update
        .filter(element.eq(updatable_element.to_string()))
        .limit(1)
        .select(LastUpdate::as_select())
        .load(connection)?;

    if let Some(result) = result.first() {
        Ok(Some(result.date()?))
    } else {
        Ok(None)
    }
}

pub(super) fn update(
    connection: &mut SqliteConnection,
    updatable_element: &UpdatableElement,
) -> Result<NaiveDateTime> {
    let update_date = Utc::now().naive_local();
    let result = diesel::update(last_update.filter(element.eq(updatable_element.to_string())))
        .set(date.eq(update_date.to_string()))
        .execute(connection)?;

    if result == 1 {
        debug!("Updated {updatable_element} at {update_date}")
    } else {
        let result = diesel::insert_into(last_update)
            .values((
                element.eq(updatable_element.to_string()),
                date.eq(update_date.to_string()),
            ))
            .execute(connection)?;

        if result != 1 {
            Err(CantUpdateLastUpdated(updatable_element.to_string()))?;
        }

        debug!("Inserted {updatable_element} at {update_date}")
    }

    Ok(update_date)
}

#[cfg(test)]
mod tests {
    use crate::database::dao::last_update::{UpdatableElement, update};
    use crate::database::model::last_update::LastUpdate;
    use crate::database::schema::last_update::dsl::last_update;
    use crate::database::schema::last_update::element;
    use chrono::NaiveDateTime;
    use diesel::prelude::*;

    fn establish_connection() -> SqliteConnection {
        crate::database::establish_connection().unwrap()
    }

    fn test_update_element(
        connection: &mut SqliteConnection,
        updatable_element: &UpdatableElement,
    ) -> NaiveDateTime {
        let update_time = update(connection, updatable_element).unwrap();

        let result = last_update
            .filter(element.eq(updatable_element.to_string()))
            .select(LastUpdate::as_select())
            .load(connection)
            .unwrap();

        assert_eq!(
            vec![LastUpdate::new(
                updatable_element.to_string().as_str(),
                update_time
            )],
            result
        );

        update_time
    }

    mod get_last_update {
        use crate::database::dao::last_update::tests::{establish_connection, test_update_element};
        use crate::database::dao::last_update::{UpdatableElement, get_last_update};
        use crate::database::with_temp_database;

        #[test]
        fn none() {
            with_temp_database(|| {
                let mut connection = establish_connection();

                assert_eq!(
                    None,
                    get_last_update(&mut connection, &UpdatableElement::Memberships).unwrap()
                );
                assert_eq!(
                    None,
                    get_last_update(&mut connection, &UpdatableElement::UdaInstances).unwrap()
                );
            })
        }

        #[test]
        fn some_after_update() {
            with_temp_database(|| {
                let mut connection = establish_connection();

                let time = test_update_element(&mut connection, &UpdatableElement::Memberships);
                assert_eq!(
                    Some(time),
                    get_last_update(&mut connection, &UpdatableElement::Memberships).unwrap()
                );
                let time = test_update_element(&mut connection, &UpdatableElement::UdaInstances);
                assert_eq!(
                    Some(time),
                    get_last_update(&mut connection, &UpdatableElement::UdaInstances).unwrap()
                );
            })
        }
    }

    mod update {
        use crate::database::dao::last_update::UpdatableElement;
        use crate::database::dao::last_update::tests::{establish_connection, test_update_element};
        use crate::database::with_temp_database;

        #[test]
        fn success_first_update() {
            with_temp_database(|| {
                let mut connection = establish_connection();
                test_update_element(&mut connection, &UpdatableElement::Memberships);
                test_update_element(&mut connection, &UpdatableElement::UdaInstances);
            })
        }

        #[test]
        fn success_second_update() {
            with_temp_database(|| {
                let mut connection = establish_connection();

                test_update_element(&mut connection, &UpdatableElement::Memberships);
                test_update_element(&mut connection, &UpdatableElement::Memberships);
                test_update_element(&mut connection, &UpdatableElement::UdaInstances);
                test_update_element(&mut connection, &UpdatableElement::UdaInstances);
            })
        }
    }
}
