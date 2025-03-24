use crate::member_identifier::MemberIdentifier;
use crate::member_to_check::MemberToCheck;
use csv::{Reader, StringRecord};
use derive_getters::Getters;
use log::warn;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// A [CsvMember] is a member whose been imported from a CSV file or string.
/// It doesn't have much information, as we want to keep it simple
/// for event organizer to check whether participants have a valid membership or not.
#[derive(Debug, Getters, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CsvMember {
    membership_num: String,
    name: Option<String>,
    first_name: Option<String>,
    identity: Option<String>,
}

impl CsvMember {
    pub fn new(
        membership_num: String,
        identity: Option<String>,
        name: Option<String>,
        first_name: Option<String>,
    ) -> Self {
        Self {
            membership_num,
            identity,
            name,
            first_name,
        }
    }

    /// Load members to check from a CSV-formatted String, such as:
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
            match Self::deserialize_member_to_check(record) {
                Ok(member) => {
                    members_to_check.insert(member);
                }
                Err(wrong_line) => {
                    if let Some(wrong_line) = wrong_line {
                        wrong_lines.push(wrong_line);
                    }
                }
            };
        });

        (members_to_check, wrong_lines)
    }

    fn deserialize_member_to_check(
        record: Result<StringRecord, csv::Error>,
    ) -> Result<CsvMember, Option<String>> {
        if let Ok(record) = record {
            let fields_count = record.len();
            if fields_count == 2 {
                Ok(CsvMember::new(
                    record.get(0).unwrap().to_owned(),
                    Some(record.get(1).unwrap().to_owned()),
                    None,
                    None,
                ))
            } else if fields_count == 3 {
                Ok(CsvMember::new(
                    record.get(0).unwrap().to_owned(),
                    None,
                    Some(record.get(1).unwrap().to_owned()),
                    Some(record.get(2).unwrap().to_owned()),
                ))
            } else {
                Err(Some(record.iter().collect::<Vec<_>>().join(";")))
            }
        } else if let Err(error) = record {
            warn!(
                "Error while deserializing member to check [error: {:?}]",
                error
            );
            Err(None)
        } else {
            Err(None)
        }
    }
}

impl MemberIdentifier for CsvMember {
    fn membership_num(&self) -> Option<String> {
        Some(self.membership_num().clone())
    }
}

impl MemberToCheck for CsvMember {
    fn id(&self) -> Option<u16> {
        None
    }

    fn identity(&self) -> Option<String> {
        self.identity.clone()
    }

    fn first_name(&self) -> Option<String> {
        self.first_name.clone()
    }

    fn last_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn email(&self) -> Option<String> {
        None
    }

    fn club(&self) -> Option<String> {
        None
    }

    fn confirmed(&self) -> Option<bool> {
        None
    }
}

impl PartialOrd for CsvMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CsvMember {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name != other.name {
            self.name.cmp(other.name())
        } else if self.first_name != other.first_name {
            self.first_name().cmp(other.first_name())
        } else {
            self.membership_num.cmp(&other.membership_num)
        }
    }
}

#[cfg(test)]
mod tests {
    mod load_members_to_check_from_csv_string {
        use crate::csv_member::CsvMember;
        use std::collections::BTreeSet;

        #[test]
        fn success_with_name_and_firstname() {
            let membership_num = "123".to_owned();
            let name = "Doe".to_owned();
            let first_name = "John".to_owned();
            let csv = format!("{membership_num};{name};{first_name}");
            let result = CsvMember::load_members_to_check_from_csv_string(&csv);
            assert_eq!(
                (
                    BTreeSet::from_iter(vec![CsvMember {
                        membership_num,
                        identity: None,
                        name: Some(name),
                        first_name: Some(first_name),
                    }]),
                    vec![]
                ),
                result
            )
        }

        #[test]
        fn success_with_identity() {
            let membership_num = "123".to_owned();
            let identity = "Doe John".to_owned();
            let csv = format!("{membership_num};{identity}");
            let result = CsvMember::load_members_to_check_from_csv_string(&csv);
            assert_eq!(
                (
                    BTreeSet::from_iter(vec![CsvMember {
                        membership_num,
                        identity: Some(identity),
                        name: None,
                        first_name: None,
                    }]),
                    vec![]
                ),
                result
            )
        }

        #[test]
        fn fail_when_wrong_row() {
            let membership_num = "123".to_owned();
            let csv = format!("{membership_num}");
            let result = CsvMember::load_members_to_check_from_csv_string(&csv);
            let expected_result = (BTreeSet::new(), vec![csv]);
            assert_eq!(expected_result, result)
        }
    }

    use crate::csv_member::CsvMember;
    use crate::member_identifier::MemberIdentifier;
    use crate::member_to_check::MemberToCheck;

    fn get_membership_number() -> String {
        "0123456789".to_owned()
    }
    fn get_identity() -> String {
        "Snow Jon".to_owned()
    }
    fn get_first_name() -> String {
        "Jon".to_owned()
    }
    fn get_last_name() -> String {
        "Snow".to_owned()
    }

    fn get_csv_member() -> CsvMember {
        CsvMember::new(
            get_membership_number(),
            Some(get_identity()),
            Some(get_last_name()),
            Some(get_first_name()),
        )
    }

    #[test]
    fn should_get_id() {
        let member = get_csv_member();
        assert_eq!(None, MemberToCheck::id(&member));
    }

    #[test]
    fn should_get_membership_number() {
        let member = get_csv_member();
        assert_eq!(
            Some(get_membership_number()),
            MemberIdentifier::membership_num(&member)
        );
    }

    #[test]
    fn should_get_identity() {
        let member = get_csv_member();
        assert_eq!(Some(get_identity()), MemberToCheck::identity(&member));
    }

    #[test]
    fn should_get_first_name() {
        let member = get_csv_member();
        assert_eq!(Some(get_first_name()), MemberToCheck::first_name(&member));
    }

    #[test]
    fn should_get_last_name() {
        let member = get_csv_member();
        assert_eq!(Some(get_last_name()), MemberToCheck::last_name(&member));
    }

    #[test]
    fn should_get_email() {
        let member = get_csv_member();
        assert_eq!(None, MemberToCheck::email(&member));
    }

    #[test]
    fn should_get_club() {
        let member = get_csv_member();
        assert_eq!(None, MemberToCheck::club(&member));
    }

    #[test]
    fn should_get_confirmed() {
        let member = get_csv_member();
        assert_eq!(None, MemberToCheck::confirmed(&member));
    }
}
