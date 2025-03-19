use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct UdaCredentials {
    /// Should be something like `https://cfm2019training.reg.unicycling-software.com`
    /// Beware of not including anything after the TLD. Otherwise, it may not work.
    uda_url: String,
    login: String,
    password: String,
}

impl UdaCredentials {
    pub fn new(uda_url: String, login: String, password: String) -> Self {
        Self {
            uda_url,
            login,
            password,
        }
    }
}
