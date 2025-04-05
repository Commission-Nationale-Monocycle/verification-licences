use crate::database::error::DatabaseError;
use chrono::NaiveDate;
use diesel::prelude::*;
use std::str::FromStr;

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = crate::database::schema::membership)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub(crate) struct Membership {
    id: i32,
    last_name: String,
    first_name: String,
    birthdate: Option<String>,
    membership_number: String,
    cell_number: Option<String>,
    email_address: String,
    start_date: String,
    end_date: String,
    club: String,
    structure_code: String,
    normalized_membership_number: String,
    normalized_last_name: String,
    normalized_first_name: String,
    normalized_last_name_first_name: String,
    normalized_first_name_last_name: String,
}

impl TryFrom<Membership> for dto::membership::Membership {
    type Error = DatabaseError;

    fn try_from(value: Membership) -> Result<Self, Self::Error> {
        let birthdate = match value.birthdate {
            Some(birthdate) => Some(NaiveDate::from_str(&birthdate)?),
            None => None,
        };
        let start_date = NaiveDate::from_str(&value.start_date)?;
        let end_date = NaiveDate::from_str(&value.end_date)?;
        Ok(dto::membership::Membership::new(
            value.last_name,
            value.first_name,
            birthdate,
            value.membership_number,
            value.cell_number,
            value.email_address,
            start_date,
            end_date,
            value.club,
            value.structure_code,
        ))
    }
}
