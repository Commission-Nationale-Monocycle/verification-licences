use crate::member_to_check::MemberToCheck;
use crate::membership::Membership;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Getters, Serialize, Deserialize, PartialEq)]
pub struct CheckedMember {
    member_to_check: MemberToCheck,
    membership: Option<Membership>,
}

impl CheckedMember {
    pub fn new(member_to_check: MemberToCheck, membership: Option<Membership>) -> Self {
        Self {
            member_to_check,
            membership,
        }
    }
}

impl PartialOrd for CheckedMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.membership.is_some() && other.membership.is_none() {
            Some(Ordering::Greater)
        } else if self.membership.is_none() && other.membership.is_some() {
            Some(Ordering::Less)
        } else {
            self.member_to_check.partial_cmp(&other.member_to_check)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::{Greater, Less};

    // region utils

    fn get_member_to_check_2() -> MemberToCheck {
        MemberToCheck::new("2".to_owned(), "".to_owned(), "".to_owned())
    }

    fn get_member_to_check_1() -> MemberToCheck {
        MemberToCheck::new("1".to_owned(), "".to_owned(), "".to_owned())
    }

    fn get_membership_2() -> Membership {
        Membership::new(
            "2".to_owned(),
            "".to_owned(),
            "".to_owned(),
            None,
            None,
            "".to_owned(),
            "".to_owned(),
            false,
            Default::default(),
            false,
            "".to_owned(),
            "".to_owned(),
        )
    }

    fn get_membership_1() -> Membership {
        Membership::new(
            "1".to_owned(),
            "".to_owned(),
            "".to_owned(),
            None,
            None,
            "".to_owned(),
            "".to_owned(),
            false,
            Default::default(),
            false,
            "".to_owned(),
            "".to_owned(),
        )
    }
    // endregion

    #[test]
    fn should_sort_by_membership() {
        let membership_1 = get_membership_1();
        let membership_2 = get_membership_2();

        let member_to_check_1 = get_member_to_check_1();
        let member_to_check_2 = get_member_to_check_2();

        let checked_member_1 = CheckedMember::new(member_to_check_1, Some(membership_1));
        let checked_member_2 = CheckedMember::new(member_to_check_2, Some(membership_2));
        assert_eq!(Some(Less), checked_member_1.partial_cmp(&checked_member_2))
    }

    #[test]
    fn should_be_greater_if_other_has_no_membership() {
        let membership_1 = get_membership_1();

        let member_to_check_1 = get_member_to_check_1();
        let member_to_check_2 = get_member_to_check_2();

        let checked_member_1 = CheckedMember::new(member_to_check_1, Some(membership_1));
        let checked_member_2 = CheckedMember::new(member_to_check_2, None);
        assert_eq!(
            Some(Greater),
            checked_member_1.partial_cmp(&checked_member_2)
        )
    }

    #[test]
    fn should_be_less_if_self_has_no_membership() {
        let membership_2 = get_membership_2();

        let member_to_check_1 = get_member_to_check_1();
        let member_to_check_2 = get_member_to_check_2();

        let checked_member_1 = CheckedMember::new(member_to_check_1, None);
        let checked_member_2 = CheckedMember::new(member_to_check_2, Some(membership_2));
        assert_eq!(Some(Less), checked_member_1.partial_cmp(&checked_member_2))
    }
}
