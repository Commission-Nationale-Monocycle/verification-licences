use std::cmp::Ordering;

use chrono::NaiveDate;
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Deserialize, Getters, PartialEq, Eq, Hash, PartialOrd, Clone)]
pub struct Member {
    #[serde(alias = "Nom d'usage")]
    name: String,
    #[serde(alias = "Prénom")]
    firstname: String,
    #[serde(alias = "Sexe")]
    gender: String,
    #[serde(alias = "Date de Naissance", deserialize_with = "date_format::deserialize_optional")]
    birthdate: Option<NaiveDate>,
    #[serde(alias = "Age")]
    age: Option<u8>,
    #[serde(alias = "Numéro d'adhérent")]
    membership_number: String,
    #[serde(alias = "Email")]
    email_address: String,
    #[serde(alias = "Réglé", deserialize_with = "bool_format::deserialize")]
    payed: bool,
    #[serde(alias = "Date Fin d'adhésion", deserialize_with = "date_format::deserialize_required")]
    end_date: NaiveDate,
    #[serde(alias = "Adherent expiré", deserialize_with = "bool_format::deserialize")]
    expired: bool,
    #[serde(alias = "Nom de structure")]
    club: String,
    #[serde(alias = "Code de structure")]
    structure_code: String,
}

impl Ord for Member {
    fn cmp(&self, other: &Self) -> Ordering {
        self.end_date.cmp(&other.end_date)
    }
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer};

    const FORMAT: &str = "%d-%m-%Y";

    pub fn deserialize_required<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(date)
    }

    pub fn deserialize_optional<'de, D>(
        deserializer: D,
    ) -> Result<Option<NaiveDate>, D::Error>
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

mod bool_format {
    use serde::{de, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<bool, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let result = match s.as_str() {
            "Oui" => Ok(true),
            "Non" => Ok(false),
            _ => Err(de::Error::unknown_variant(&s, &["Oui", "Non"]))
        };

        result
    }
}