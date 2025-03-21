use crate::member_identifier::MemberIdentifier;
use crate::member_to_check::MemberToCheck;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// An [UdaMember] is a participant imported from UDA.
/// It has a few fields, which can help to manage this member - confirm them, email them, ...
#[derive(Debug, Getters, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct UdaMember {
    id: u16,
    membership_number: Option<String>,
    first_name: String,
    last_name: String,
    email: String,
    club: Option<String>,
    confirmed: bool,
}

impl UdaMember {
    pub fn new(
        id: u16,
        membership_number: Option<String>,
        first_name: String,
        last_name: String,
        email: String,
        club: Option<String>,
        confirmed: bool,
    ) -> Self {
        Self {
            id,
            membership_number,
            first_name,
            last_name,
            email,
            club,
            confirmed,
        }
    }
}

impl PartialOrd for UdaMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UdaMember {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.last_name() != other.last_name() {
            self.last_name().cmp(other.last_name())
        } else if self.first_name() != other.first_name() {
            self.first_name().cmp(other.first_name())
        } else {
            self.membership_num().cmp(&other.membership_num())
        }
    }
}

impl MemberIdentifier for UdaMember {
    fn membership_num(&self) -> Option<String> {
        self.membership_number.clone()
    }
}

impl MemberToCheck for UdaMember {
    fn id(&self) -> Option<u16> {
        Some(self.id)
    }

    fn first_name(&self) -> String {
        self.first_name.clone()
    }

    fn last_name(&self) -> String {
        self.last_name.clone()
    }

    fn email(&self) -> Option<String> {
        Some(self.email.clone())
    }

    fn club(&self) -> Option<String> {
        self.club.clone()
    }

    fn confirmed(&self) -> Option<bool> {
        Some(self.confirmed)
    }
}
