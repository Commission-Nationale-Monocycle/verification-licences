use crate::member_identifier::MemberIdentifier;
use crate::member_to_check::MemberToCheck;
use csv::Reader;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// A [CsvMember] is a member whose been imported from a CSV file or string.
/// It doesn't have much information, as we want to keep it simple
/// for event organizer to check whether participants have a valid membership or not.
#[derive(Debug, Getters, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CsvMember {
    membership_num: String,
    name: String,
    first_name: String,
}

impl CsvMember {
    pub fn new(membership_num: String, name: String, first_name: String) -> Self {
        Self {
            membership_num,
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
            if let Ok(record) = record {
                if record.len() != 3 {
                    wrong_lines.push(record.iter().collect::<Vec<_>>().join(";"));
                } else {
                    members_to_check.insert(CsvMember::new(
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

impl MemberIdentifier for CsvMember {
    fn membership_num(&self) -> Option<String> {
        Some(self.membership_num().clone())
    }
}

impl MemberToCheck for CsvMember {
    fn id(&self) -> Option<u16> {
        None
    }

    fn first_name(&self) -> String {
        self.first_name.clone()
    }

    fn last_name(&self) -> String {
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
        fn success() {
            let membership_num = "123".to_owned();
            let name = "Doe".to_owned();
            let first_name = "John".to_owned();
            let csv = format!("{membership_num};{name};{first_name}");
            let result = CsvMember::load_members_to_check_from_csv_string(&csv);
            assert_eq!(
                (
                    BTreeSet::from_iter(vec![CsvMember {
                        membership_num,
                        name,
                        first_name
                    }]),
                    vec![]
                ),
                result
            )
        }

        #[test]
        fn fail_when_wrong_row() {
            let membership_num = "123".to_owned();
            let name = "Doe".to_owned();
            let csv = format!("{membership_num};{name}");
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
    fn get_first_name() -> String {
        "Jon".to_owned()
    }
    fn get_last_name() -> String {
        "Snow".to_owned()
    }

    fn get_csv_member() -> CsvMember {
        CsvMember::new(get_membership_number(), get_last_name(), get_first_name())
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
    fn should_get_first_name() {
        let member = get_csv_member();
        assert_eq!(get_first_name(), MemberToCheck::first_name(&member));
    }

    #[test]
    fn should_get_last_name() {
        let member = get_csv_member();
        assert_eq!(get_last_name(), MemberToCheck::last_name(&member));
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
