use std::collections::HashMap;
use std::fmt::Debug;

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
