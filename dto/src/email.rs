use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize)]
pub struct Email {
    recipients: Vec<String>,
    subject: String,
    body: String,
}

impl Email {
    pub fn new(recipients: Vec<String>, subject: String, body: String) -> Self {
        Self {
            recipients,
            subject,
            body,
        }
    }
}
