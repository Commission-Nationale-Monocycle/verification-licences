use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::member::memberships::Memberships;

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Members {
    members: HashMap<String, Memberships>
}

impl Deref for Members {
    type Target = HashMap<String, Memberships>;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl From<HashMap<String, Memberships>> for Members {
    fn from(members: HashMap<String, Memberships>) -> Self {
        Members { members }
    }
}

/// (membership_num, last_name, first_name)
type MemberToCheck<'a> = (&'a str, &'a str, &'a str);

pub fn check_members<'a>(members: &Members, members_to_check: &'a [MemberToCheck<'a>]) -> Vec<(&'a MemberToCheck<'a>, bool)> {
    members_to_check
        .iter()
        .map(|member_to_check| (member_to_check, check_member(members, member_to_check)))
        .collect()
}

fn check_member<'a>(members: &Members, member_to_check: &'a MemberToCheck<'a>) -> bool {
    let membership_num = member_to_check.0;

    members.iter()
        .any(|member| member.0 == membership_num)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::member::memberships::Memberships;
    use crate::member::members::{check_member, check_members, Members};
    use crate::member::tests::{get_expected_member, MEMBER_FIRSTNAME, MEMBER_NAME, MEMBERSHIP_NUMBER};

    // region check_members
    #[test]
    fn members_should_be_checked() {
        let member = get_expected_member();
        let members = Members::from(HashMap::from([(MEMBERSHIP_NUMBER.to_string(), Memberships::from([member]))]));
        let member_to_check = (MEMBERSHIP_NUMBER, MEMBER_NAME, MEMBER_FIRSTNAME);

        assert_eq!(vec![(&member_to_check, true)], check_members(&members, &[member_to_check]));
    }

    #[test]
    fn members_should_not_be_checked() {
        let member = get_expected_member();
        let members = Members::from(HashMap::from([(MEMBERSHIP_NUMBER.to_string(), Memberships::from([member]))]));
        let membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = (membership_number.as_str(), MEMBER_NAME, MEMBER_FIRSTNAME);

        assert_eq!(vec![(&member_to_check, false)], check_members(&members, &[member_to_check]));
    }
    // endregion

    // region check_member
    #[test]
    fn member_should_be_check() {
        let member = get_expected_member();
        let members = Members::from(HashMap::from([(MEMBERSHIP_NUMBER.to_string(), Memberships::from([member]))]));
        let member_to_check = (MEMBERSHIP_NUMBER, MEMBER_NAME, MEMBER_FIRSTNAME);

        assert!(check_member(&members, &member_to_check));
    }

    #[test]
    fn member_should_not_be_check() {
        let member = get_expected_member();
        let members = Members::from(HashMap::from([(MEMBERSHIP_NUMBER.to_string(), Memberships::from([member]))]));
        let membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = (membership_number.as_str(), MEMBER_NAME, MEMBER_FIRSTNAME);

        assert!(!check_member(&members, &member_to_check));
    }
    // endregion
}