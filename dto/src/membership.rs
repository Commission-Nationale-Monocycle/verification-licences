use chrono::NaiveDate;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq, Eq, Hash, Clone)]
pub struct Membership {
    membership_number: String,
    name: String,
    first_name: String,
    birthdate: Option<NaiveDate>,
    cell_number: Option<String>,
    email_address: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    club: String,
    structure_code: String,
}

impl Membership {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        first_name: String,
        birthdate: Option<NaiveDate>,
        membership_number: String,
        cell_number: Option<String>,
        email_address: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        club: String,
        structure_code: String,
    ) -> Self {
        Self {
            membership_number,
            name,
            first_name,
            birthdate,
            cell_number,
            email_address,
            start_date,
            end_date,
            club,
            structure_code,
        }
    }
}

impl PartialOrd for Membership {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Membership {
    fn cmp(&self, other: &Self) -> Ordering {
        self.end_date
            .cmp(&other.end_date)
            .then(self.membership_number.cmp(&other.membership_number))
            .then(self.name.cmp(&other.name))
            .then(self.first_name.cmp(&other.first_name))
    }
}

#[cfg(any(test, feature = "test"))]
pub mod tests {
    use super::*;
    use chrono::Months;
    use parameterized::{ide, parameterized};

    ide!();

    impl Membership {
        pub fn new_test(end_date: NaiveDate) -> Self {
            Membership {
                name: "".to_string(),
                first_name: "".to_string(),
                birthdate: None,
                cell_number: None,
                membership_number: "".to_string(),
                email_address: "".to_string(),
                start_date: end_date.checked_sub_months(Months::new(12)).unwrap(),
                end_date,
                club: "".to_string(),
                structure_code: "".to_string(),
            }
        }
    }

    const HEADER: &str = "Nom d'usage;Prénom;Date de Naissance;Numéro d'adhérent;Téléphone portable;Email;Date Début d'adhésion;Date Fin d'adhésion;Nom de structure;Code de structure";
    const MEMBERSHIP_AS_CSV: &str = "Doe;Jon;01-02-1980;123456;+33 6 12 34 56 78;email@address.com;30-09-2024;30-09-2025;My club;Z01234";
    pub const MEMBER_NAME: &str = "Doe";
    pub const MEMBER_FIRST_NAME: &str = "Jon";
    pub const MEMBERSHIP_NUMBER: &str = "123456";
    const MALFORMED_MEMBERSHIP_AS_CSV: &str =
        "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oops;30-09-2025;Non;My club;Z01234";

    pub fn get_expected_membership() -> Membership {
        Membership {
            name: "Doe".to_string(),
            first_name: "Jon".to_string(),
            birthdate: NaiveDate::from_ymd_opt(1980, 2, 1),
            membership_number: MEMBERSHIP_NUMBER.to_string(),
            cell_number: Some("+33 6 12 34 56 78".to_string()),
            email_address: "email@address.com".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            club: "My club".to_string(),
            structure_code: "Z01234".to_string(),
        }
    }

    pub fn get_membership_as_csv() -> String {
        format!("{HEADER}\n{MEMBERSHIP_AS_CSV}")
    }

    pub fn get_malformed_membership_as_csv() -> String {
        format!("{HEADER}\n{MALFORMED_MEMBERSHIP_AS_CSV}")
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
    fn should_sort_memberships(
        end_dates: ((i32, u32, u32), (i32, u32, u32)),
        expected_result: Ordering,
    ) {
        let ((y1, m1, d1), (y2, m2, d2)) = end_dates;
        let member1 = Membership::new_test(NaiveDate::from_ymd_opt(y1, m1, d1).unwrap());
        let member2 = Membership::new_test(NaiveDate::from_ymd_opt(y2, m2, d2).unwrap());
        assert_eq!(Some(expected_result), member1.partial_cmp(&member2));
    }
}
