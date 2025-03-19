use crate::member_identifier::MemberIdentifier;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Getters, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Participant {
    id: u16,
    membership_number: Option<String>,
    first_name: String,
    last_name: String,
    email: String,
    club: Option<String>,
    confirmed: bool,
}

impl Participant {
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

impl PartialOrd for Participant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Participant {
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

impl MemberIdentifier for Participant {
    fn membership_num(&self) -> Option<String> {
        self.membership_number.clone()
    }
}
