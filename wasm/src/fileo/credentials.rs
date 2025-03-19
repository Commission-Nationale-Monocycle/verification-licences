use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct FileoCredentials {
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

impl FileoCredentials {
    pub fn new(login: String, password: String) -> Self {
        Self { login, password }
    }
}
