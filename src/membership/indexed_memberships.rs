use crate::membership::memberships::Memberships;
use crate::tools::normalize;
use derive_getters::Getters;
use dto::membership::Membership;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

/// [Memberships] indexed by different keys:
/// - Membership number
/// - (Last name, first name)
/// - Identity.
///
/// Can be dereferenced into [Memberships].
/// As of now, it stores copies of memberships and can thus take a lot of memory.
/// This may have to be optimized later on.
#[derive(Getters, Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct IndexedMemberships {
    memberships: Memberships,
    memberships_by_num: HashMap<String, Memberships>,
    memberships_by_name: HashMap<(String, String), Memberships>, // Indexed by (last name, first name)
    memberships_by_identity: HashMap<String, Memberships>,
}

impl Deref for IndexedMemberships {
    type Target = Memberships;

    fn deref(&self) -> &Self::Target {
        &self.memberships
    }
}

impl From<Vec<Membership>> for IndexedMemberships {
    fn from(value: Vec<Membership>) -> Self {
        let memberships = value.clone().into();

        let mut memberships_by_num = group_by(&value, |membership| {
            normalize(membership.membership_number())
        });
        memberships_by_num.extend(group_by(
            &value
                .iter()
                .flat_map(|membership| {
                    if membership.membership_number().parse::<u32>().is_ok() {
                        Some(membership.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            |membership| {
                membership
                    .membership_number()
                    .parse::<u32>()
                    .expect("Should be a u32")
                    .to_string()
            },
        ));

        let mut memberships_by_name = group_by(&value, |membership| {
            (
                normalize(membership.name()),
                normalize(membership.first_name()),
            )
        });
        // People may invert their first and last name... So we try both.
        memberships_by_name.extend(group_by(&value, |membership| {
            (
                normalize(membership.first_name()),
                normalize(membership.name()),
            )
        }));

        let mut memberships_by_identity = group_by(&value, |membership| {
            format!(
                "{}{}",
                normalize(membership.name()),
                normalize(membership.first_name()),
            )
        });

        memberships_by_identity.extend(group_by(&value, |membership| {
            format!(
                "{}{}",
                normalize(membership.first_name()),
                normalize(membership.name()),
            )
        }));

        Self {
            memberships,
            memberships_by_num,
            memberships_by_name,
            memberships_by_identity,
        }
    }
}

fn group_by<T, F>(memberships: &[Membership], f: F) -> HashMap<T, Memberships>
where
    T: Eq + Hash + Debug,
    F: Fn(&&Membership) -> T,
{
    let mut map = HashMap::<T, Memberships>::new();
    for membership in memberships {
        let key = f(&membership);
        map.entry(key).or_default().insert(membership.clone());
    }

    map
}

#[cfg(test)]
mod tests {
    mod from_vec {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::memberships::Memberships;
        use crate::tools::normalize;
        use chrono::NaiveDate;
        use dto::membership::Membership;
        use std::collections::HashMap;

        #[test]
        fn successfully_sorted() {
            let jon_doe = jon_doe();
            let jon_doe_previous_membership = jon_doe_previous_membership();
            let jonette_snow = jonette_snow();
            let other_jon_doe = other_jon_doe();
            let memberships = vec![
                jon_doe.clone(),
                jonette_snow.clone(),
                jon_doe_previous_membership.clone(),
                other_jon_doe.clone(),
            ];

            let expected_memberships = Memberships::from(memberships.clone());
            let expected_grouped_by_num = {
                let mut map = HashMap::new();

                map.insert(
                    jon_doe.membership_number().to_owned(),
                    Memberships::from(vec![jon_doe.clone(), jon_doe_previous_membership.clone()]),
                );

                map.insert(
                    jonette_snow.membership_number().to_owned(),
                    Memberships::from(vec![jonette_snow.clone()]),
                );

                map.insert(
                    other_jon_doe.membership_number().to_owned(),
                    Memberships::from(vec![other_jon_doe.clone()]),
                );

                map
            };

            let expected_grouped_by_name = {
                let mut map = HashMap::new();

                map.insert(
                    (normalize(jon_doe.name()), normalize(jon_doe.first_name())),
                    Memberships::from(vec![
                        jon_doe.clone(),
                        jon_doe_previous_membership.clone(),
                        other_jon_doe.clone(),
                    ]),
                );
                map.insert(
                    (normalize(jon_doe.first_name()), normalize(jon_doe.name())),
                    Memberships::from(vec![
                        jon_doe.clone(),
                        jon_doe_previous_membership.clone(),
                        other_jon_doe.clone(),
                    ]),
                );

                map.insert(
                    (
                        normalize(jonette_snow.name()),
                        normalize(jonette_snow.first_name()),
                    ),
                    Memberships::from(vec![jonette_snow.clone()]),
                );
                map.insert(
                    (
                        normalize(jonette_snow.first_name()),
                        normalize(jonette_snow.name()),
                    ),
                    Memberships::from(vec![jonette_snow.clone()]),
                );

                map
            };

            let expected_grouped_by_identity = {
                let mut map = HashMap::new();

                map.insert(
                    format!(
                        "{}{}",
                        normalize(jon_doe.name()),
                        normalize(jon_doe.first_name())
                    ),
                    Memberships::from(vec![
                        jon_doe.clone(),
                        jon_doe_previous_membership.clone(),
                        other_jon_doe.clone(),
                    ]),
                );
                map.insert(
                    format!(
                        "{}{}",
                        normalize(jon_doe.first_name()),
                        normalize(jon_doe.name())
                    ),
                    Memberships::from(vec![
                        jon_doe.clone(),
                        jon_doe_previous_membership.clone(),
                        other_jon_doe.clone(),
                    ]),
                );

                map.insert(
                    format!(
                        "{}{}",
                        normalize(jonette_snow.name()),
                        normalize(jonette_snow.first_name())
                    ),
                    Memberships::from(vec![jonette_snow.clone()]),
                );
                map.insert(
                    format!(
                        "{}{}",
                        normalize(jonette_snow.first_name()),
                        normalize(jonette_snow.name())
                    ),
                    Memberships::from(vec![jonette_snow.clone()]),
                );

                map
            };

            let indexed_memberships = IndexedMemberships::from(memberships);
            assert_eq!(expected_memberships, indexed_memberships.memberships);
            assert_eq!(
                expected_grouped_by_num,
                indexed_memberships.memberships_by_num
            );
            dbg!(&indexed_memberships.memberships_by_name);
            assert_eq!(
                expected_grouped_by_name,
                indexed_memberships.memberships_by_name
            );
            assert_eq!(
                expected_grouped_by_identity,
                indexed_memberships.memberships_by_identity
            );
        }

        fn jonette_snow() -> Membership {
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

        fn jon_doe() -> Membership {
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

        fn jon_doe_previous_membership() -> Membership {
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

        fn other_jon_doe() -> Membership {
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
}
