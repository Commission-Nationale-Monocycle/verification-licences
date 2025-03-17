use crate::tools::log_error_and_return;
use crate::web::credentials_storage::CredentialsStorage;
use rocket::State;
use rocket::http::{Cookie, Status};
use rocket::outcome::{Outcome, try_outcome};
use rocket::request::{self, Request};
use std::sync::Mutex;

/// Retrieve credentials based on a cookie.
/// If no credentials are associated to the cookie, or if no such cookie is present in the request,
/// then returns a Forawrd outcome containing an Unauthorized status. This lets other routes to take on the request.
/// Otherwise, return the retrieved credentials as a Success outcome.
pub async fn from_request<C: Send + Sync + Clone + 'static>(
    req: &Request<'_>,
    cookie_name: &str,
) -> request::Outcome<C, ()> {
    if let Some(cookie) = get_authentication_cookie(req, cookie_name) {
        let credentials_storage =
            try_outcome!(req.guard::<&State<Mutex<CredentialsStorage<C>>>>().await);
        match credentials_storage.lock() {
            Ok(credentials_storage) => match credentials_storage.get(cookie.value()) {
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
