use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Getters, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    slug: String,
    name: String,
    url: String,
}

impl Instance {
    pub fn new(slug: String, name: String, url: String) -> Self {
        Self { slug, name, url }
    }
}
