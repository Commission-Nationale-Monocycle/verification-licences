use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// A simple wrapper around [uda_connector::credentials::UdaCredentials].
/// This is required to implement traits on this struct.
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct UdaCredentials {
    credentials: uda_connector::credentials::UdaCredentials,
}

impl Deref for UdaCredentials {
    type Target = uda_connector::credentials::UdaCredentials;

    fn deref(&self) -> &Self::Target {
        &self.credentials
    }
}

impl DerefMut for UdaCredentials {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.credentials
    }
}

impl From<uda_connector::credentials::UdaCredentials> for UdaCredentials {
    fn from(credentials: uda_connector::credentials::UdaCredentials) -> Self {
        Self { credentials }
    }
}
