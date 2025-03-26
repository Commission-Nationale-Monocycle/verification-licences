use crate::membership::memberships::Memberships;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

// FIXME: delete this struct; it isn't useful anymore.
/// A map of [Memberships], grouped by membership number.
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct GroupedMemberships {
    memberships: HashMap<String, Memberships>,
}

impl Deref for GroupedMemberships {
    type Target = HashMap<String, Memberships>;

    fn deref(&self) -> &Self::Target {
        &self.memberships
    }
}

impl From<HashMap<String, Memberships>> for GroupedMemberships {
    fn from(members: HashMap<String, Memberships>) -> Self {
        GroupedMemberships {
            memberships: members,
        }
    }
}
