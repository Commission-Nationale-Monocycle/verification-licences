use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

use derive_getters::Getters;
use rocket::serde::{Deserialize, Serialize};

use crate::member::MemberDto;

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Getters)]
pub struct Memberships {
    memberships: BTreeSet<MemberDto>,
}

impl Deref for Memberships {
    type Target = BTreeSet<MemberDto>;

    fn deref(&self) -> &Self::Target {
        &self.memberships
    }
}

impl DerefMut for Memberships {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memberships
    }
}

impl<const N: usize> From<[MemberDto; N]> for Memberships {
    fn from(value: [MemberDto; N]) -> Self {
        Memberships { memberships: BTreeSet::from(value)}
    }
}

impl Memberships {
    pub fn find_last_membership(&self) -> Option<&MemberDto> {
        self.memberships.iter().max()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use chrono::NaiveDate;

    use crate::member::memberships::Memberships;
    use crate::member::MemberDto;

    #[test]
    fn should_retrieve_last_membership() {
        let first_end_date = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        let second_end_date = NaiveDate::from_ymd_opt(2025, 9, 30).unwrap();
        let first_membership = MemberDto::new_test(first_end_date);
        let second_membership = MemberDto::new_test(second_end_date);

        let memberships = Memberships::from([first_membership.clone(), second_membership.clone()]);
        assert_eq!(Some(&second_membership), memberships.find_last_membership());
        assert_eq!(2, memberships.deref().len());
        assert!(memberships.contains(&first_membership));
        assert!(memberships.contains(&second_membership));
    }

    #[test]
    fn should_not_retrieve_last_membership_as_no_membership() {
        let memberships = Memberships::from([]);
        assert_eq!(None, memberships.find_last_membership());
        assert_eq!(0, memberships.deref().len());
    }
}