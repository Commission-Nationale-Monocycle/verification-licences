use crate::checked_member::MemberStatus::{Expired, Unknown, UpToDate};
use crate::member_identifier::MemberIdentifier;
use crate::membership::Membership;
use chrono::Utc;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
pub enum MemberStatus {
    UpToDate,
    Expired,
    Unknown,
}

#[derive(Debug, Getters, Serialize, Deserialize, PartialEq)]
pub struct CheckedMember<T: MemberIdentifier> {
    member_to_check: T,
    membership: Option<Membership>,
}

impl<T: MemberIdentifier> CheckedMember<T> {
    pub fn new(member_to_check: T, membership: Option<Membership>) -> Self {
        Self {
            member_to_check,
            membership,
        }
    }

    pub fn compute_member_status(&self) -> MemberStatus {
        match &self.membership {
            None => Unknown,
            Some(membership) => {
                if Utc::now().date_naive() <= *membership.end_date() {
                    UpToDate
                } else {
                    Expired
                }
            }
        }
    }
}

impl<T: MemberIdentifier> PartialOrd for CheckedMember<T> {
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
    use crate::member_to_check::MemberToCheck;
    use chrono::Days;
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

    // region compute_member_status
    #[test]
    fn should_be_up_to_date() {
        let membership = Membership::new(
            "1".to_owned(),
            "".to_owned(),
            "".to_owned(),
            None,
            None,
            "".to_owned(),
            "".to_owned(),
            false,
            Utc::now()
                .date_naive()
                .checked_add_days(Days::new(10))
                .unwrap(),
            false,
            "".to_owned(),
            "".to_owned(),
        );

        let checked_member = CheckedMember::new(get_member_to_check_1(), Some(membership));
        assert_eq!(UpToDate, checked_member.compute_member_status());
    }

    #[test]
    fn should_be_expired() {
        let membership = Membership::new(
            "1".to_owned(),
            "".to_owned(),
            "".to_owned(),
            None,
            None,
            "".to_owned(),
            "".to_owned(),
            false,
            Utc::now()
                .date_naive()
                .checked_sub_days(Days::new(10))
                .unwrap(),
            false,
            "".to_owned(),
            "".to_owned(),
        );

        let checked_member = CheckedMember::new(get_member_to_check_1(), Some(membership));
        assert_eq!(Expired, checked_member.compute_member_status());
    }

    #[test]
    fn should_be_unknown() {
        let checked_member = CheckedMember::new(get_member_to_check_1(), None);
        assert_eq!(Unknown, checked_member.compute_member_status());
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
