use derive_getters::Getters;

#[derive(Debug, Getters)]
pub struct Configuration {
    instances_list_url: String,
}

impl Configuration {
    pub fn new(instances_list_url: String) -> Self {
        Self { instances_list_url }
    }
}
