use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

use derive_getters::Getters;
use dto::membership::Membership;
use rocket::serde::{Deserialize, Serialize};

/// A sorted list of unique [Membership]s.
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Getters)]
pub struct Memberships {
    memberships: BTreeSet<Membership>,
}

impl Deref for Memberships {
    type Target = BTreeSet<Membership>;

    fn deref(&self) -> &Self::Target {
        &self.memberships
    }
}

impl DerefMut for Memberships {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memberships
    }
}

impl<const N: usize> From<[Membership; N]> for Memberships {
    fn from(value: [Membership; N]) -> Self {
        Memberships {
            memberships: BTreeSet::from(value),
        }
    }
}

impl From<Vec<Membership>> for Memberships {
    fn from(value: Vec<Membership>) -> Self {
        Self {
            memberships: value.into_iter().collect(),
        }
    }
}

impl Extend<Membership> for Memberships {
    fn extend<T: IntoIterator<Item = Membership>>(&mut self, iter: T) {
        for membership in iter {
            self.memberships.insert(membership);
        }
    }
}

impl Memberships {
    pub fn find_last_membership(&self) -> Option<&Membership> {
        self.memberships.iter().max()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use chrono::NaiveDate;

    use crate::membership::memberships::Memberships;
    use dto::membership::Membership;

    #[test]
    fn should_retrieve_last_membership() {
        let first_end_date = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        let second_end_date = NaiveDate::from_ymd_opt(2025, 9, 30).unwrap();
        let first_membership = Membership::new_test(first_end_date);
        let second_membership = Membership::new_test(second_end_date);

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
