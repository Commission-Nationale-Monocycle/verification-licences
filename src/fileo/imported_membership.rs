use chrono::NaiveDate;
use derive_getters::Getters;
use dto::membership::Membership;
use rocket::serde::Deserialize;

/// A membership as retrieved from Fileo.
/// As all fields are in French and are sometimes formatted in a strange way,
/// it is required to add a few annotations.
#[derive(Debug, Deserialize, Getters, PartialEq, Eq, Hash, Clone)]
pub struct ImportedMembership {
    #[serde(alias = "Nom d'usage")]
    name: String,
    #[serde(alias = "Prénom")]
    first_name: String,
    #[serde(
        alias = "Date de Naissance",
        deserialize_with = "date_format::deserialize_optional"
    )]
    birthdate: Option<NaiveDate>,
    #[serde(alias = "Numéro d'adhérent")]
    membership_number: String,
    #[serde(
        alias = "Téléphone portable",
        deserialize_with = "optional_string_format::deserialize"
    )]
    cell_num: Option<String>,
    #[serde(alias = "Email")]
    email_address: String,
    #[serde(
        alias = "Date Début d'adhésion",
        deserialize_with = "date_format::deserialize_required"
    )]
    start_date: NaiveDate,
    #[serde(
        alias = "Date Fin d'adhésion",
        deserialize_with = "date_format::deserialize_required"
    )]
    end_date: NaiveDate,
    #[serde(alias = "Nom de structure")]
    club: String,
    #[serde(alias = "Code de structure")]
    structure_code: String,
}

impl From<ImportedMembership> for Membership {
    fn from(membership: ImportedMembership) -> Self {
        Membership::new(
            membership.name,
            membership.first_name,
            membership.birthdate,
            membership.membership_number,
            membership.cell_num,
            membership.email_address,
            membership.start_date,
            membership.end_date,
            membership.club,
            membership.structure_code,
        )
    }
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer};

    const FORMAT: &str = "%d-%m-%Y";

    pub fn deserialize_required<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(date)
    }

    pub fn deserialize_optional<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.trim().is_empty() {
            Ok(None)
        } else {
            let date = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
            Ok(Some(date))
        }
    }
}

mod optional_string_format {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(s.trim().to_owned()))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::fileo::imported_membership::ImportedMembership;
    use chrono::NaiveDate;
    use parameterized::ide;
    use rocket::serde::json;

    ide!();

    #[test]
    fn should_deserialize_member() {
        let membership = ImportedMembership {
            name: "Doe".to_owned(),
            first_name: "John".to_owned(),
            birthdate: NaiveDate::from_ymd_opt(2000, 10, 11),
            membership_number: "42".to_string(),
            cell_num: Some("+33 6 12 34 56 78".to_owned()),
            email_address: "john.doe@yopmail.com".to_owned(),
            start_date: NaiveDate::from_ymd_opt(2024, 10, 11).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 10, 11).unwrap(),
            club: "Best Club".to_owned(),
            structure_code: "A12345".to_owned(),
        };
        let json = r#"{"Nom d'usage":"Doe","Prénom":"John","Date de Naissance":"11-10-2000","Numéro d'adhérent":"42","Téléphone portable":"+33 6 12 34 56 78","Email":"john.doe@yopmail.com","Date Début d'adhésion":"11-10-2024","Date Fin d'adhésion":"11-10-2025","Nom de structure":"Best Club","Code de structure":"A12345"}"#;
        let result = json::from_str(json);

        assert!(result.is_ok());
        assert_eq!(membership, result.unwrap())
    }

    #[test]
    fn should_deserialize_when_empty_date() {
        let membership = ImportedMembership {
            name: "Doe".to_owned(),
            first_name: "John".to_owned(),
            birthdate: None,
            membership_number: "42".to_string(),
            cell_num: None,
            email_address: "john.doe@yopmail.com".to_owned(),
            start_date: NaiveDate::from_ymd_opt(2024, 10, 11).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 10, 11).unwrap(),
            club: "Best Club".to_owned(),
            structure_code: "A12345".to_owned(),
        };
        let json = r#"{"Nom d'usage":"Doe","Prénom":"John","Date de Naissance":"","Numéro d'adhérent":"42","Téléphone portable":"","Email":"john.doe@yopmail.com","Date Début d'adhésion":"11-10-2024","Date Fin d'adhésion":"11-10-2025","Nom de structure":"Best Club","Code de structure":"A12345"}"#;
        let result = json::from_str(json);

        assert!(result.is_ok());
        assert_eq!(membership, result.unwrap())
    }
}
