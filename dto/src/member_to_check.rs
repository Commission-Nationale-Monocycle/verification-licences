use crate::member_identifier::MemberIdentifier;
use csv::Reader;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Debug, Getters, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MemberToCheck {
    membership_num: String,
    name: String,
    firstname: String,
}

impl MemberToCheck {
    pub fn new(membership_num: String, name: String, firstname: String) -> Self {
        Self {
            membership_num,
            name,
            firstname,
        }
    }
}

impl MemberIdentifier for MemberToCheck {
    fn membership_num(&self) -> Option<String> {
        Some(self.membership_num().clone())
    }
}

impl MemberToCheck {
    /// Load members to check rom a CSV-formatted String, such as:
    /// `membership_num;name;firstname`
    pub fn load_members_to_check_from_csv_string(
        members_to_check: &str,
    ) -> (BTreeSet<Self>, Vec<String>) {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(true)
            .from_reader(members_to_check.as_bytes());

        Self::load_members_to_check_from_csv(&mut reader)
    }

    /// Load members to check rom a CSV-formatted Reader, such as:
    /// `membership_num;name;firstname`
    fn load_members_to_check_from_csv<T>(reader: &mut Reader<T>) -> (BTreeSet<Self>, Vec<String>)
    where
        T: std::io::Read,
    {
        let mut members_to_check = BTreeSet::new();
        let mut wrong_lines = vec![];

        reader.records().for_each(|record| {
            if let Ok(record) = record {
                if record.len() != 3 {
                    wrong_lines.push(record.iter().collect::<Vec<_>>().join(";"));
                } else {
                    members_to_check.insert(MemberToCheck::new(
                        record.get(0).unwrap().to_owned(),
                        record.get(1).unwrap().to_owned(),
                        record.get(2).unwrap().to_owned(),
                    ));
                }
            };
        });

        (members_to_check, wrong_lines)
    }
}

impl PartialOrd for MemberToCheck {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemberToCheck {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name != other.name {
            self.name.cmp(other.name())
        } else if self.firstname != other.firstname {
            self.firstname().cmp(other.firstname())
        } else {
            self.membership_num.cmp(&other.membership_num)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::member_to_check::MemberToCheck;
    use std::collections::BTreeSet;

    // region load_members_to_check_from_csv_string
    #[test]
    fn should_load_members_to_check_from_csv_string() {
        let membership_num = "123".to_owned();
        let name = "Doe".to_owned();
        let firstname = "John".to_owned();
        let csv = format!("{membership_num};{name};{firstname}");
        let result = MemberToCheck::load_members_to_check_from_csv_string(&csv);
        assert_eq!(
            (
                BTreeSet::from_iter(vec![MemberToCheck {
                    membership_num,
                    name,
                    firstname
                }]),
                vec![]
            ),
            result
        )
    }

    #[test]
    fn should_not_load_members_to_check_from_csv_string_when_wrong_row() {
        let membership_num = "123".to_owned();
        let name = "Doe".to_owned();
        let csv = format!("{membership_num};{name}");
        let result = MemberToCheck::load_members_to_check_from_csv_string(&csv);
        let expected_result = (BTreeSet::new(), vec![csv]);
        assert_eq!(expected_result, result)
    }
    // endregion
}
