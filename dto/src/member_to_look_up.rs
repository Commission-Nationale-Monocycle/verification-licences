use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Debug, Serialize, Deserialize)]
pub struct MemberToLookUp {
    membership_num: Option<String>,
    last_name: Option<String>,
    first_name: Option<String>,
}

impl MemberToLookUp {
    pub fn new(
        membership_num: Option<String>,
        last_name: Option<String>,
        first_name: Option<String>,
    ) -> Self {
        Self {
            membership_num,
            last_name,
            first_name,
        }
    }
}
