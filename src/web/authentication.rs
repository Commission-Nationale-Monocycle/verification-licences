use crate::tools::log_error_and_return;
use crate::web::credentials::{Credentials, CredentialsStorage};
use rocket::State;
use rocket::http::Status;
use rocket::outcome::{Outcome, try_outcome};
use rocket::request::{self, FromRequest, Request};
use std::sync::Mutex;

const AUTHENTICATION_HEADER: &str = "Fileo-Authentication";

/// If an endpoint requires a Fileo credential to be called,
/// then its implementation should require a [Credentials] parameter.
/// Rocket will summon this guard to ensure such credentials exists.
/// If it doesn't, then the caller receives an Unauthorized status.
///
/// Currently, such authentication is passed from the caller to the server using a `Fileo-Authentication` header.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Credentials {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(uuid) = req.headers().get_one(AUTHENTICATION_HEADER) {
            let credentials_storage =
                try_outcome!(req.guard::<&State<Mutex<CredentialsStorage>>>().await);
            match credentials_storage.lock() {
                Ok(credentials_storage) => match credentials_storage.get(uuid) {
                    None => Outcome::Error((Status::Unauthorized, ())),
                    Some(credentials) => Outcome::Success(credentials.clone()),
                },
                Err(error) => {
                    log_error_and_return(Outcome::Error((Status::InternalServerError, ())))(error)
                }
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Header;
    use rocket::local::asynchronous::Client;

    #[async_test]
    async fn should_request_succeed() {
        let credentials = Credentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let credentials_uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store(credentials_uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let header = Header::new(AUTHENTICATION_HEADER, credentials_uuid);
        let request = client.get("http://localhost").header(header);

        let outcome = Credentials::from_request(&request).await;
        assert!(outcome.is_success())
    }

    #[async_test]
    async fn should_request_fail_when_no_matching_credentials() {
        let credentials_storage = CredentialsStorage::default();
        let credentials_uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let header = Header::new(AUTHENTICATION_HEADER, credentials_uuid);
        let request = client.get("http://localhost").header(header);

        let outcome = Credentials::from_request(&request).await;
        assert!(outcome.is_error());
        assert_eq!(Status::Unauthorized, outcome.failed().unwrap().0);
    }

    #[async_test]
    async fn should_request_fail_when_no_header() {
        let credentials_storage = CredentialsStorage::default();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("http://localhost");

        let outcome = Credentials::from_request(&request).await;
        assert!(outcome.is_error());
        assert_eq!(Status::Unauthorized, outcome.failed().unwrap().0);
    }
}
