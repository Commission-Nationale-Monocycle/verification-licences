use crate::member_to_check::MemberToCheck;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A [CsvMember] is a member whose been imported from a CSV file or string.
/// It doesn't have much information, as we want to keep it simple
/// for event organizer to check whether participants have a valid membership or not.
#[derive(Debug, Getters, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CsvMember {
    membership_num: Option<String>,
    identity: Option<String>,
    name: Option<String>,
    first_name: Option<String>,
}

impl CsvMember {
    pub fn new(
        membership_num: Option<String>,
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
}

impl MemberToCheck for CsvMember {
    fn id(&self) -> Option<u16> {
        None
    }

    fn membership_num(&self) -> Option<String> {
        self.membership_num().clone()
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
    use crate::csv_member::CsvMember;
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
            Some(get_membership_number()),
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
            MemberToCheck::membership_num(&member)
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
