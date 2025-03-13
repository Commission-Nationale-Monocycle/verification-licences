use crate::tools::log_error_and_return;
use crate::web::credentials::{CredentialsStorage, FileoCredentials, UdaCredentials};
use rocket::State;
use rocket::http::{Cookie, Status};
use rocket::outcome::{Outcome, try_outcome};
use rocket::request::{self, FromRequest, Request};
use std::sync::Mutex;

pub const FILEO_AUTHENTICATION_COOKIE: &str = "Fileo-Authentication";
pub const UDA_AUTHENTICATION_COOKIE: &str = "UDA-Authentication";

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
        if let Some(cookie) = get_authentication_cookie(req, FILEO_AUTHENTICATION_COOKIE) {
            let credentials_storage =
                try_outcome!(req.guard::<&State<Mutex<CredentialsStorage>>>().await);
            match credentials_storage.lock() {
                Ok(credentials_storage) => match credentials_storage.get_fileo(cookie.value()) {
                    None => Outcome::Forward(Status::Unauthorized),
                    Some(credentials) => Outcome::Success(credentials.clone()),
                },
                Err(error) => {
                    log_error_and_return(Outcome::Error((Status::InternalServerError, ())))(error)
                }
            }
        } else {
            Outcome::Forward(Status::Unauthorized)
        }
    }
}

/// If an endpoint requires UDA credentials to be called,
/// then its implementation should require a [UdaCredentials] parameter.
/// Rocket will summon this guard to ensure such credentials exists.
/// If it doesn't, then the caller receives an Unauthorized status.
///
/// Currently, such authentication is passed from the caller to the server using a `UDA-Authentication` private cookie.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UdaCredentials {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = get_authentication_cookie(req, UDA_AUTHENTICATION_COOKIE) {
            let credentials_storage =
                try_outcome!(req.guard::<&State<Mutex<CredentialsStorage>>>().await);
            match credentials_storage.lock() {
                Ok(credentials_storage) => match credentials_storage.get_uda(cookie.value()) {
                    None => Outcome::Forward(Status::Unauthorized),
                    Some(credentials) => Outcome::Success(credentials.clone()),
                },
                Err(error) => {
                    log_error_and_return(Outcome::Error((Status::InternalServerError, ())))(error)
                }
            }
        } else {
            Outcome::Forward(Status::Unauthorized)
        }
    }
}

#[cfg(not(test))]
fn get_authentication_cookie<'a>(req: &'a Request, cookie_name: &str) -> Option<Cookie<'a>> {
    req.cookies().get_private(cookie_name)
}

/// For tests, we have to ensure the cookie is there, pending or not. Otherwise, it doesn't work.
/// Thus, the need to hijack the normal method.
#[cfg(test)]
fn get_authentication_cookie<'a>(req: &'a Request, cookie_name: &str) -> Option<Cookie<'a>> {
    req.cookies().get_pending(cookie_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::credentials::FileoCredentials;
    use rocket::http::Cookie;
    use rocket::local::asynchronous::Client;

    #[async_test]
    async fn should_request_succeed() {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store_fileo(uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(FILEO_AUTHENTICATION_COOKIE, uuid);
        let request = client.get("http://localhost").cookie(cookie.clone());

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_success());
    }

    #[async_test]
    async fn should_request_fail_when_no_matching_credentials() {
        let credentials_storage = CredentialsStorage::default();
        let credentials_uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(FILEO_AUTHENTICATION_COOKIE, credentials_uuid);
        let request = client.get("http://localhost").cookie(cookie);

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_forward());
        assert_eq!(Status::Unauthorized, outcome.forwarded().unwrap());
    }

    #[async_test]
    async fn should_request_fail_when_no_header() {
        let credentials_storage = CredentialsStorage::default();
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build().manage(credentials_storage_mutex);
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("http://localhost");

        let outcome = FileoCredentials::from_request(&request).await;
        assert!(outcome.is_forward());
        assert_eq!(Status::Unauthorized, outcome.forwarded().unwrap());
    }
}
