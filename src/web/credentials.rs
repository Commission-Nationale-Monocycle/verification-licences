use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone)]
pub struct Credentials {
    login: String,
    password: String,
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
}

#[cfg(test)]
impl Credentials {
    pub fn new(login: String, password: String) -> Self {
        Self { login, password }
    }
}
