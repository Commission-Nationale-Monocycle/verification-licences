use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone, Default)]
pub struct FileoCredentials {
    login: String,
    password: String,
}

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone, Default)]
pub struct UdaCredentials {
    /// Should be something like `https://cfm2019training.reg.unicycling-software.com`
    /// Beware of not including anything after the TLD. Otherwise, it may not work.
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

#[derive(Default, Debug)]
pub struct CredentialsStorage<C: Send + Sync> {
    credentials: HashMap<String, C>,
}

impl<C: Send + Sync> CredentialsStorage<C> {
    pub fn store(&mut self, id: String, credentials: C) {
        self.credentials.insert(id, credentials);
    }

    pub fn get(&self, id: &str) -> Option<&C> {
        self.credentials.get(id)
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
