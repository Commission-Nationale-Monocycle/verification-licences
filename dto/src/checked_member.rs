use crate::member_to_check::MemberToCheck;
use crate::membership::Membership;
use crate::membership_status::MemberStatus::Unknown;
use crate::membership_status::{MemberStatus, compute_member_status};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum CheckResult {
    Match(Membership),
    PartialMatch(Membership),
    NoMatch,
}

/// Ordering is based on whether there are match.
/// Put simply, Match is greater than Partial Match, which in turn is greater than NoMatch.
/// If both self & other have the same level, then it is based on the memberships themselves.
impl PartialOrd for CheckResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ordering is based on whether there are match.
/// Put simply, Match is greater than Partial Match, which in turn is greater than NoMatch.
/// If both self & other have the same level, then it is based on the memberships themselves.
impl Ord for CheckResult {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            CheckResult::Match(self_membership) => match other {
                CheckResult::Match(other_membership) => self_membership.cmp(other_membership),
                _ => Ordering::Greater,
            },
            CheckResult::PartialMatch(self_membership) => match other {
                CheckResult::Match(_) => Ordering::Less,
                CheckResult::PartialMatch(other_membership) => {
                    self_membership.cmp(other_membership)
                }
                CheckResult::NoMatch => Ordering::Greater,
            },
            CheckResult::NoMatch => match other {
                CheckResult::NoMatch => Ordering::Equal,
                _ => Ordering::Less,
            },
        }
    }
}

/// A [CheckedMember] is a member whose membership has been checked.
/// It may have a membership up-to-date, an expired membership or no membership at all.
#[derive(Debug, Getters, Serialize, Deserialize, PartialEq)]
pub struct CheckedMember<T: MemberToCheck> {
    member_to_check: T,
    membership: CheckResult,
}

impl<T: MemberToCheck> CheckedMember<T> {
    pub fn new(member_to_check: T, membership: CheckResult) -> Self {
        Self {
            member_to_check,
            membership,
        }
    }

    pub fn compute_member_status(&self) -> MemberStatus {
        match &self.membership {
            CheckResult::NoMatch => Unknown,
            CheckResult::Match(membership) | CheckResult::PartialMatch(membership) => {
                compute_member_status(Some(membership))
            }
        }
    }
}

impl<T: MemberToCheck> PartialOrd for CheckedMember<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let memberships_order = self.membership.cmp(&other.membership);
        if memberships_order != Ordering::Equal {
            Some(memberships_order)
        } else {
            self.member_to_check.partial_cmp(&other.member_to_check)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_member::CsvMember;

    // region utils
    fn get_member_to_check_2() -> CsvMember {
        CsvMember::new(
            Some("2".to_owned()),
            None,
            Some("".to_owned()),
            Some("".to_owned()),
        )
    }

    fn get_member_to_check_1() -> CsvMember {
        CsvMember::new(
            Some("1".to_owned()),
            None,
            Some("".to_owned()),
            Some("".to_owned()),
        )
    }

    fn get_membership_2() -> Membership {
        Membership::new(
            "2".to_owned(),
            "".to_owned(),
            None,
            "".to_owned(),
            None,
            "".to_owned(),
            Default::default(),
            Default::default(),
            "".to_owned(),
            "".to_owned(),
        )
    }

    fn get_membership_1() -> Membership {
        Membership::new(
            "1".to_owned(),
            "".to_owned(),
            None,
            "".to_owned(),
            None,
            "".to_owned(),
            Default::default(),
            Default::default(),
            "".to_owned(),
            "".to_owned(),
        )
    }
    // endregion

    mod check_result_cmp {
        use crate::checked_member::CheckResult::{Match, NoMatch, PartialMatch};
        use crate::checked_member::tests::{get_membership_1, get_membership_2};
        use std::cmp::Ordering;

        #[test]
        fn match_match_same_membership_is_equal() {
            let membership = get_membership_1();
            assert_eq!(
                Ordering::Equal,
                Match(membership.clone()).cmp(&Match(membership.clone()))
            );
        }

        #[test]
        fn partial_match_partial_match_same_membership_is_equal() {
            let membership = get_membership_1();
            assert_eq!(
                Ordering::Equal,
                PartialMatch(membership.clone()).cmp(&PartialMatch(membership.clone()))
            );
        }

        #[test]
        fn no_match_no_match_is_equal() {
            assert_eq!(Ordering::Equal, NoMatch.cmp(&NoMatch));
        }

        #[test]
        fn match_partial_match_is_greater() {
            let membership_1 = get_membership_1();
            let membership_2 = get_membership_2();
            assert_eq!(
                Ordering::Greater,
                Match(membership_1).cmp(&PartialMatch(membership_2))
            );
        }

        #[test]
        fn match_no_match_is_greater() {
            let membership = get_membership_1();
            assert_eq!(Ordering::Greater, Match(membership).cmp(&NoMatch));
        }

        #[test]
        fn partial_match_match_is_less() {
            let membership_1 = get_membership_1();
            let membership_2 = get_membership_2();
            assert_eq!(
                Ordering::Less,
                PartialMatch(membership_1).cmp(&Match(membership_2))
            );
        }

        #[test]
        fn partial_match_no_match_is_greater() {
            let membership_ = get_membership_1();
            assert_eq!(Ordering::Greater, PartialMatch(membership_).cmp(&NoMatch));
        }

        #[test]
        fn no_match_match_is_less() {
            let membership = get_membership_1();
            assert_eq!(Ordering::Less, NoMatch.cmp(&Match(membership)));
        }

        #[test]
        fn no_match_partial_match_is_less() {
            let membership = get_membership_1();
            assert_eq!(Ordering::Less, NoMatch.cmp(&PartialMatch(membership)));
        }

        #[test]
        fn match_match_same_membership_is_membership_ord() {
            let membership_1 = get_membership_1();
            let membership_2 = get_membership_2();
            assert_eq!(
                membership_1.cmp(&membership_2),
                Match(membership_1.clone()).cmp(&Match(membership_2.clone()))
            );
        }

        #[test]
        fn partial_match_partial_match_same_membership_is_membership_ord() {
            let membership_1 = get_membership_1();
            let membership_2 = get_membership_2();
            assert_eq!(
                membership_1.cmp(&membership_2),
                PartialMatch(membership_1.clone()).cmp(&PartialMatch(membership_2.clone()))
            );
        }
    }

    mod compute_member_status {
        use crate::checked_member::tests::get_member_to_check_1;
        use crate::checked_member::{CheckResult, CheckedMember};
        use crate::membership::Membership;
        use crate::membership_status::MemberStatus::{Expired, Unknown, UpToDate};
        use chrono::{Days, Months, Utc};

        #[test]
        fn should_be_up_to_date() {
            let membership = Membership::new(
                "1".to_owned(),
                "".to_owned(),
                None,
                "".to_owned(),
                None,
                "".to_owned(),
                Utc::now()
                    .date_naive()
                    .checked_sub_months(Months::new(12))
                    .unwrap(),
                Utc::now()
                    .date_naive()
                    .checked_add_days(Days::new(10))
                    .unwrap(),
                "".to_owned(),
                "".to_owned(),
            );

            let checked_member =
                CheckedMember::new(get_member_to_check_1(), CheckResult::Match(membership));
            assert_eq!(UpToDate, checked_member.compute_member_status());
        }

        #[test]
        fn should_be_expired() {
            let membership = Membership::new(
                "1".to_owned(),
                "".to_owned(),
                None,
                "".to_owned(),
                None,
                "".to_owned(),
                Utc::now()
                    .date_naive()
                    .checked_sub_months(Months::new(12))
                    .unwrap(),
                Utc::now()
                    .date_naive()
                    .checked_sub_days(Days::new(10))
                    .unwrap(),
                "".to_owned(),
                "".to_owned(),
            );

            let checked_member =
                CheckedMember::new(get_member_to_check_1(), CheckResult::Match(membership));
            assert_eq!(Expired, checked_member.compute_member_status());
        }

        #[test]
        fn should_be_unknown() {
            let checked_member = CheckedMember::new(get_member_to_check_1(), CheckResult::NoMatch);
            assert_eq!(Unknown, checked_member.compute_member_status());
        }
    }

    mod checked_member_partial_cmp {
        use crate::checked_member::tests::{
            get_member_to_check_1, get_member_to_check_2, get_membership_1, get_membership_2,
        };
        use crate::checked_member::{CheckResult, CheckedMember};
        use std::cmp::Ordering::{Greater, Less};

        #[test]
        fn should_sort_by_membership() {
            let membership_1 = get_membership_1();
            let membership_2 = get_membership_2();

            let member_to_check_1 = get_member_to_check_1();
            let member_to_check_2 = get_member_to_check_2();

            let checked_member_1 =
                CheckedMember::new(member_to_check_1, CheckResult::Match(membership_1));
            let checked_member_2 =
                CheckedMember::new(member_to_check_2, CheckResult::Match(membership_2));
            assert_eq!(Some(Less), checked_member_1.partial_cmp(&checked_member_2))
        }

        #[test]
        fn should_be_greater_if_other_has_no_membership() {
            let membership_1 = get_membership_1();

            let member_to_check_1 = get_member_to_check_1();
            let member_to_check_2 = get_member_to_check_2();

            let checked_member_1 =
                CheckedMember::new(member_to_check_1, CheckResult::Match(membership_1));
            let checked_member_2 = CheckedMember::new(member_to_check_2, CheckResult::NoMatch);
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

            let checked_member_1 = CheckedMember::new(member_to_check_1, CheckResult::NoMatch);
            let checked_member_2 =
                CheckedMember::new(member_to_check_2, CheckResult::Match(membership_2));
            assert_eq!(Some(Less), checked_member_1.partial_cmp(&checked_member_2))
        }
    }
}
