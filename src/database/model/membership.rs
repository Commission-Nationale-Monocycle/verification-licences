use crate::database::error::DatabaseError;
use chrono::NaiveDate;
use diesel::prelude::*;
use std::str::FromStr;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::database::schema::membership)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub(crate) struct Membership {
    id: i32,
    last_name: String,
    first_name: String,
    gender: String,
    birthdate: Option<String>,
    age: Option<i32>,
    membership_number: String,
    email_address: String,
    payed: bool,
    end_date: String,
    expired: bool,
    club: String,
    structure_code: String,
}

impl TryFrom<Membership> for dto::membership::Membership {
    type Error = DatabaseError;

    fn try_from(value: Membership) -> Result<Self, Self::Error> {
        let birthdate = match value.birthdate {
            Some(birthdate) => Some(NaiveDate::from_str(&birthdate)?),
            None => None,
        };
        let end_date = NaiveDate::from_str(&value.end_date)?;
        Ok(dto::membership::Membership::new(
            value.last_name,
            value.first_name,
            value.gender,
            birthdate,
            value.age.map(|age| age as u8),
            value.membership_number,
            value.email_address,
            value.payed,
            end_date,
            value.expired,
            value.club,
            value.structure_code,
        ))
    }
}
