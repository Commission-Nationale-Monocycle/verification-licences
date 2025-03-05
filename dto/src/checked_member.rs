use crate::member_to_check::MemberToCheck;
use crate::membership::Membership;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Getters, Serialize, Deserialize, PartialEq)]
pub struct CheckedMember {
    member_to_check: MemberToCheck,
    membership_dto: Option<Membership>,
}

impl CheckedMember {
    pub fn new(member_to_check: MemberToCheck, membership_dto: Option<Membership>) -> Self {
        Self {
            member_to_check,
            membership_dto,
        }
    }
}

impl PartialOrd for CheckedMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.membership_dto.is_some() && other.membership_dto.is_none() {
            Some(Ordering::Greater)
        } else if self.membership_dto.is_none() && other.membership_dto.is_some() {
            Some(Ordering::Less)
        } else {
            self.member_to_check.partial_cmp(&other.member_to_check)
        }
    }
}
