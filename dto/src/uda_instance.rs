use chrono::{NaiveDate, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Getters, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Getters, Default, PartialEq, Serialize, Deserialize)]
pub struct InstancesList {
    instances: Vec<Instance>,
    update_date: Option<NaiveDate>,
}

impl InstancesList {
    pub fn set_instances(&mut self, instances: Vec<Instance>) {
        self.instances = instances;
        self.update_date = Some(Utc::now().date_naive());
    }
}
