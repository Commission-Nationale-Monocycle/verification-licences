use crate::database::error::DatabaseError;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq)]
#[diesel(table_name = crate::database::schema::last_update)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub(crate) struct LastUpdate {
    element: String,
    date: String,
}

impl LastUpdate {
    #[cfg(test)]
    pub(crate) fn new(element: &str, date: NaiveDateTime) -> Self {
        Self {
            element: element.to_string(),
            date: date.to_string(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn date(&self) -> Result<NaiveDateTime, DatabaseError> {
        NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S%.f")
            .map_err(DatabaseError::from)
    }
}
