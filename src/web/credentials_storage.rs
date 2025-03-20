use cached::{Cached, TimedSizedCache};
use std::fmt::Debug;

const CACHE_SIZE: usize = 100;

/// A container for storing credentials. Only 100 credentials can be stored at a time,
/// and they expire after one month.
#[derive(Debug)]
pub struct CredentialsStorage<C: Send + Sync> {
    credentials: TimedSizedCache<String, C>,
}

impl<C: Send + Sync> CredentialsStorage<C> {
    pub fn store(&mut self, id: String, credentials: C) {
        self.credentials.cache_set(id, credentials);
    }

    pub fn get(&mut self, id: &str) -> Option<&C> {
        self.credentials.cache_get(id)
    }
}

impl<C: Send + Sync> Default for CredentialsStorage<C> {
    /// By default, credentials expire after one month.
    fn default() -> Self {
        let credentials = TimedSizedCache::with_size_and_lifespan(CACHE_SIZE, 60 * 60 * 24 * 30);
        Self { credentials }
    }
}

#[cfg(test)]
mod tests {
    use crate::web::credentials_storage::CredentialsStorage;
    use cached::Cached;

    #[test]
    fn should_store_only_100_credentials() {
        let mut storage: CredentialsStorage<()> = CredentialsStorage::default();
        assert_eq!(0, storage.credentials.cache_size());
        (0..100).for_each(|id| storage.store(id.to_string(), ()));
        (0..100).for_each(|id| assert_eq!(Some(&()), storage.get(&id.to_string())));
        assert_eq!(100, storage.credentials.cache_size());
        storage.store("100".to_owned(), ());
        assert_eq!(100, storage.credentials.cache_size());
        assert_eq!(None, storage.get("0"));
    }
}
