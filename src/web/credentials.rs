use derive_getters::Getters;
use serde::Deserialize;
use std::collections::HashMap;
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

#[derive(Default)]
pub struct CredentialsStorage {
    credentials: HashMap<String, Credentials>,
}

impl CredentialsStorage {
    pub fn store(&mut self, id: String, credentials: Credentials) {
        self.credentials.insert(id, credentials);
    }

    pub fn get(&self, id: &str) -> Option<&Credentials> {
        self.credentials.get(id)
    }

    pub fn is_logged_in(&self, id: &str) -> bool {
        self.credentials.contains_key(id)
    }
}
