use crate::fileo::credentials::FileoCredentials;
use crate::web::authentication;
use rocket::request::FromRequest;
use rocket::{Request, request};

pub const AUTHENTICATION_COOKIE: &str = "Fileo-Authentication";

/// If an endpoint requires Fileo credentials to be called,
/// then its implementation should require a [FileoCredentials] parameter.
/// Rocket will summon this guard to ensure such credentials exists.
/// If it doesn't, then the caller receives an Unauthorized status.
///
/// Currently, such authentication is passed from the caller to the server using a `Fileo-Authentication` private cookie.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for FileoCredentials {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        authentication::from_request(req, AUTHENTICATION_COOKIE).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileo::credentials::FileoCredentials;
    use crate::web::credentials_storage::CredentialsStorage;
    use rocket::http::{Cookie, Status};
    use rocket::local::asynchronous::Client;
    use std::sync::Mutex;

    #[async_test]
    async fn should_fileo_request_succeed() {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store(uuid.clone(), credentials.clone());
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);
        let request = client.get("http://localhost").cookie(cookie.clone());

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_success());
        assert_eq!(credentials, outcome.succeeded().unwrap());
    }

    #[async_test]
    async fn should_fileo_request_fail_when_no_matching_credentials() {
        let credentials_storage = CredentialsStorage::<FileoCredentials>::default();
        let credentials_uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(AUTHENTICATION_COOKIE, credentials_uuid);
        let request = client.get("http://localhost").cookie(cookie);

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_forward());
        assert_eq!(Status::Unauthorized, outcome.forwarded().unwrap());
    }

    #[async_test]
    async fn should_fileo_request_fail_when_no_header() {
        let credentials_storage = CredentialsStorage::<FileoCredentials>::default();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("http://localhost");

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_forward());
        assert_eq!(Status::Unauthorized, outcome.forwarded().unwrap());
    }
}
