use derive_getters::Getters;
use rocket::serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Getters, PartialEq, Clone, Default)]
pub struct UdaCredentials {
    /// Should be something like `https://cfm2019training.reg.unicycling-software.com`
    /// Beware of not including anything after the TLD. Otherwise, it may not work.
    #[getter(skip)]
    uda_url: String,
    login: String,
    password: String,
}

impl UdaCredentials {
    #[cfg(not(feature = "demo"))]
    pub fn uda_url(&self) -> &String {
        &self.uda_url
    }

    #[cfg(feature = "demo")]
    pub fn uda_url(&self) -> &String {
        crate::demo_mock_server::UDA_MOCK_SERVER_URI.get().unwrap()
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
