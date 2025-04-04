pub(crate) mod check;
pub(crate) mod config;
pub(crate) mod look_up;
pub(crate) mod memberships;

#[cfg(test)]
pub(crate) mod tests {
    use chrono::NaiveDate;
    use dto::membership::Membership;

    pub(crate) fn jonette_snow() -> Membership {
        Membership::new(
            "Snow".to_string(),
            "Jonette".to_string(),
            "F".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            Some(72),
            "654321".to_string(),
            "jonette.snow@address.com".to_string(),
            true,
            NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
            false,
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn jon_doe() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            "H".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            Some(45),
            "123456".to_string(),
            "jon.doe@address.com".to_string(),
            true,
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            false,
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn jon_doe_previous_membership() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            "H".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            Some(45),
            "123456".to_string(),
            "jon.doe@address.com".to_string(),
            true,
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            false,
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }

    pub(crate) fn other_jon_doe() -> Membership {
        Membership::new(
            "Doe".to_string(),
            "Jon".to_string(),
            "H".to_string(),
            NaiveDate::from_ymd_opt(1990, 11, 5),
            Some(45),
            "897654".to_string(),
            "jon.doe@address.com".to_string(),
            true,
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            false,
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }
}
