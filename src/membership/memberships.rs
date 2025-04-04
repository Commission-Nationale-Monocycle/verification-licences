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
