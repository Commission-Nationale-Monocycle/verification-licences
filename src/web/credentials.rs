use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

// FIXME: find a way to factorize this quick and dirty implementation

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone)]
pub struct FileoCredentials {
    login: String,
    password: String,
}

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone)]
pub struct UdaCredentials {
    uda_url: String,
    login: String,
    password: String,
}

impl Debug for FileoCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Fileo Credentials {{login={}, password=MASKED}}",
            self.login
        )
    }
}

impl Debug for UdaCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Uda Credentials {{uda={}, login={}, password=MASKED}}",
            self.password, self.login
        )
    }
}

#[derive(Default)]
pub struct CredentialsStorage {
    // FIXME: should be clear at some point to avoid memory leaks
    fileo_credentials: HashMap<String, FileoCredentials>,
    uda_credentials: HashMap<String, UdaCredentials>,
}

impl CredentialsStorage {
    pub fn store_fileo(&mut self, id: String, credentials: FileoCredentials) {
        self.fileo_credentials.insert(id, credentials);
    }

    pub fn store_uda(&mut self, id: String, credentials: UdaCredentials) {
        self.uda_credentials.insert(id, credentials);
    }

    pub fn get_fileo(&self, id: &str) -> Option<&FileoCredentials> {
        self.fileo_credentials.get(id)
    }

    pub fn get_uda(&self, id: &str) -> Option<&UdaCredentials> {
        self.uda_credentials.get(id)
    }
}

#[cfg(test)]
impl FileoCredentials {
    pub fn new(login: String, password: String) -> Self {
        Self { login, password }
    }
}

#[cfg(test)]
impl UdaCredentials {
    pub fn new(uda_url: String, login: String, password: String) -> Self {
        Self {
            uda_url,
            login,
            password,
        }
    }
}
