use std::cmp::Ordering;
use std::ffi::OsStr;

use chrono::NaiveDate;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::member::error::Error;

pub mod download;
pub mod import_from_file;
pub mod file_details;
pub mod error;
pub mod config;
pub mod members;
pub mod memberships;

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(not(feature = "demo"))]
const MEMBERS_FILE_FOLDER: &str = "data";
#[cfg(feature = "demo")]
const MEMBERS_FILE_FOLDER: &str = "demo_data";
pub fn get_members_file_folder() -> &'static OsStr {
    MEMBERS_FILE_FOLDER.as_ref()
}

#[derive(Debug, Deserialize, Getters, PartialEq, Eq, Hash, Clone)]
pub struct Membership {
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

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq, Eq, Hash, Clone)]
pub struct MembershipDto {
    name: String,
    firstname: String,
    gender: String,
    birthdate: Option<NaiveDate>,
    age: Option<u8>,
    membership_number: String,
    email_address: String,
    payed: bool,
    end_date: NaiveDate,
    expired: bool,
    club: String,
    structure_code: String,
}

impl PartialOrd for MembershipDto {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MembershipDto {
    fn cmp(&self, other: &Self) -> Ordering {
        self.membership_number.cmp(&other.membership_number)
            .then(self.name.cmp(&other.name))
            .then(self.firstname.cmp(&other.firstname))
            .then(self.end_date.cmp(&other.end_date))
    }
}

impl From<Membership> for MembershipDto {
    fn from(membership: Membership) -> Self {
        MembershipDto {
            name: membership.name,
            firstname: membership.firstname,
            gender: membership.gender,
            birthdate: membership.birthdate,
            age: membership.age,
            membership_number: membership.membership_number,
            email_address: membership.email_address,
            payed: membership.payed,
            end_date: membership.end_date,
            expired: membership.expired,
            club: membership.club,
            structure_code: membership.structure_code,
        }
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

#[cfg(test)]
pub mod tests {
    use std::cmp::Ordering;

    use chrono::NaiveDate;
    use parameterized::ide;
    use parameterized::parameterized;

    use crate::member::{Membership, MembershipDto};

    ide!();

    impl MembershipDto {
        pub fn new_test(end_date: NaiveDate) -> Self {
            MembershipDto {
                name: "".to_string(),
                firstname: "".to_string(),
                gender: "".to_string(),
                birthdate: None,
                age: None,
                membership_number: "".to_string(),
                email_address: "".to_string(),
                payed: false,
                end_date,
                expired: false,
                club: "".to_string(),
                structure_code: "".to_string(),
            }
        }
    }

    const HEADER: &str = "Nom d'usage;Prénom;Sexe;Date de Naissance;Age;Numéro d'adhérent;Email;Réglé;Date Fin d'adhésion;Adherent expiré;Nom de structure;Code de structure";
    const MEMBER_AS_CSV: &str = "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oui;30-09-2025;Non;My club;Z01234";
    pub const MEMBER_NAME: &str = "Doe";
    pub const MEMBER_FIRSTNAME: &str = "Jon";
    pub const MEMBERSHIP_NUMBER: &str = "123456";
    const MALFORMED_MEMBER_AS_CSV: &str = "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oops;30-09-2025;Non;My club;Z01234";

    pub fn get_expected_member() -> MembershipDto {
        MembershipDto {
            name: "Doe".to_string(),
            firstname: "Jon".to_string(),
            gender: "H".to_string(),
            birthdate: NaiveDate::from_ymd_opt(1980, 2, 1),
            age: Some(45),
            membership_number: MEMBERSHIP_NUMBER.to_string(),
            email_address: "email@address.com".to_string(),
            payed: true,
            end_date: NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            expired: false,
            club: "My club".to_string(),
            structure_code: "Z01234".to_string(),
        }
    }

    pub fn get_member_as_csv() -> String {
        format!("{HEADER}\n{MEMBER_AS_CSV}")
    }

    pub fn get_malformed_member_as_csv() -> String {
        format!("{HEADER}\n{MALFORMED_MEMBER_AS_CSV}")
    }

    #[parameterized(
        end_dates = {
        ((2020, 10, 12), (2020, 11, 12)),
        ((2020, 11, 12), (2020, 10, 12)),
        ((2020, 11, 12), (2020, 11, 12)),
        },
        expected_result = {
        Ordering::Less,
        Ordering::Greater,
        Ordering::Equal,
        }
    )]
    fn should_sort_members(end_dates: ((i32, u32, u32), (i32, u32, u32)), expected_result: Ordering) {
        let ((y1, m1, d1), (y2, m2, d2)) = end_dates;
        let member1 = MembershipDto::new_test(NaiveDate::from_ymd_opt(y1, m1, d1).unwrap());
        let member2 = MembershipDto::new_test(NaiveDate::from_ymd_opt(y2, m2, d2).unwrap());
        assert_eq!(Some(expected_result), member1.partial_cmp(&member2));
    }

    #[test]
    fn should_deserialize_member() {
        let membership = Membership {
            name: "Doe".to_owned(),
            firstname: "John".to_owned(),
            gender: "M".to_string(),
            birthdate: NaiveDate::from_ymd_opt(2000, 10, 11),
            age: Some(24_u8),
            membership_number: "42".to_string(),
            email_address: "john.doe@yopmail.com".to_owned(),
            payed: true,
            end_date: NaiveDate::from_ymd_opt(2025, 10, 11).unwrap(),
            expired: false,
            club: "Best Club".to_owned(),
            structure_code: "A12345".to_owned(),
        };
        let json = r#"{"Nom d'usage":"Doe","Prénom":"John","Sexe":"M","Date de Naissance":"11-10-2000","Age":24,"Numéro d'adhérent":"42","Email":"john.doe@yopmail.com","Réglé":"Oui","Date Fin d'adhésion":"11-10-2025","Adherent expiré":"Non","Nom de structure":"Best Club","Code de structure":"A12345"}"#;
        let result = serde_json::from_str(json);

        assert!(result.is_ok());
        assert_eq!(membership, result.unwrap())
    }

    #[test]
    fn should_deserialize_when_empty_date() {
        let membership = Membership {
            name: "Doe".to_owned(),
            firstname: "John".to_owned(),
            gender: "M".to_string(),
            birthdate: None,
            age: None,
            membership_number: "42".to_string(),
            email_address: "john.doe@yopmail.com".to_owned(),
            payed: true,
            end_date: NaiveDate::from_ymd_opt(2025, 10, 11).unwrap(),
            expired: false,
            club: "Best Club".to_owned(),
            structure_code: "A12345".to_owned(),
        };
        let json = r#"{"Nom d'usage":"Doe","Prénom":"John","Sexe":"M","Date de Naissance":"","Numéro d'adhérent":"42","Email":"john.doe@yopmail.com","Réglé":"Oui","Date Fin d'adhésion":"11-10-2025","Adherent expiré":"Non","Nom de structure":"Best Club","Code de structure":"A12345"}"#;
        let result = serde_json::from_str(json);

        assert!(result.is_ok());
        assert_eq!(membership, result.unwrap())
    }


    #[parameterized(
        payed = {"Oops", ""}
    )]
    fn should_not_deserialize_member_as_wrong_bool(payed: &str) {
        let json = format!(r#"{{"Nom d'usage":"Doe","Prénom":"John","Sexe":"M","Date de Naissance":"11-10-2000","Age":24,"Numéro d'adhérent":"42","Email":"john.doe@yopmail.com","Réglé":"{payed}","Date Fin d'adhésion":"11-10-2025","Adherent expiré":"Non","Nom de structure":"Best Club","Code de structure":"A12345"}}"#);
        let result: Result<Membership, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}
