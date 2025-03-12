use derive_getters::Getters;
use serde::Deserialize;
use std::fmt::{Debug, Formatter};

#[derive(Deserialize, Getters, PartialEq)]
pub struct Credentials {
    login: String,
    password: String,
}

impl Credentials {
    pub fn new(login: String, password: String) -> Self {
        Self { login, password }
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Credentials {{login={}, password=MASKED}}", self.login)
    }
}
