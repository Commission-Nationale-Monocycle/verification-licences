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

    fn identity(&self) -> Option<String> {
        Some(format!("{} {}", self.first_name, self.last_name))
    }

    fn first_name(&self) -> Option<String> {
        Some(self.first_name.clone())
    }

    fn last_name(&self) -> Option<String> {
        Some(self.last_name.clone())
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

#[cfg(test)]
mod tests {
    use crate::member_identifier::MemberIdentifier;
    use crate::member_to_check::MemberToCheck;
    use crate::uda_member::UdaMember;

    fn get_id() -> u16 {
        42
    }
    fn get_membership_number() -> Option<String> {
        Some("0123456789".to_owned())
    }
    fn get_first_name() -> String {
        "Jon".to_owned()
    }
    fn get_last_name() -> String {
        "Snow".to_owned()
    }
    fn get_email() -> String {
        "jon.snow@email.com".to_owned()
    }
    fn get_club() -> Option<String> {
        Some("My club".to_owned())
    }
    fn get_confirmed() -> bool {
        true
    }

    fn get_uda_member() -> UdaMember {
        UdaMember::new(
            get_id(),
            get_membership_number(),
            get_first_name(),
            get_last_name(),
            get_email(),
            get_club(),
            get_confirmed(),
        )
    }

    #[test]
    fn should_get_id() {
        let member = get_uda_member();
        assert_eq!(Some(get_id()), MemberToCheck::id(&member));
    }

    #[test]
    fn should_get_membership_number() {
        let member = get_uda_member();
        assert_eq!(
            get_membership_number(),
            MemberIdentifier::membership_num(&member)
        );
    }

    #[test]
    fn should_get_first_name() {
        let member = get_uda_member();
        assert_eq!(Some(get_first_name()), MemberToCheck::first_name(&member));
    }

    #[test]
    fn should_get_last_name() {
        let member = get_uda_member();
        assert_eq!(Some(get_last_name()), MemberToCheck::last_name(&member));
    }

    #[test]
    fn should_get_email() {
        let member = get_uda_member();
        assert_eq!(Some(get_email()), MemberToCheck::email(&member));
    }

    #[test]
    fn should_get_club() {
        let member = get_uda_member();
        assert_eq!(get_club(), MemberToCheck::club(&member));
    }

    #[test]
    fn should_get_confirmed() {
        let member = get_uda_member();
        assert_eq!(Some(get_confirmed()), MemberToCheck::confirmed(&member));
    }
}
