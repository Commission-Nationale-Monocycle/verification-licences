pub(crate) mod check;
pub(crate) mod config;
pub(crate) mod look_up;

#[cfg(test)]
pub(crate) mod tests {
    use chrono::NaiveDate;
    use dto::membership::Membership;

    pub(crate) fn jonette_snow() -> Membership {
        Membership::new(
            "Snow".to_string(),
            "Jonette".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            "654321".to_string(),
            None,
            "jonette.snow@address.com".to_string(),
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn jon_doe() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            "123456".to_string(),
            None,
            "jon.doe@address.com".to_string(),
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn jon_doe_previous_membership() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            "123456".to_string(),
            None,
            "jon.doe@address.com".to_string(),
            NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn other_jon_doe() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            NaiveDate::from_ymd_opt(1990, 11, 5),
            "897654".to_string(),
            None,
            "jon.doe@address.com".to_string(),
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }
}
