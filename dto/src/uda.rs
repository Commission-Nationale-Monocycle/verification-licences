use derive_getters::Getters;

#[derive(Debug, Getters, PartialEq)]
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
