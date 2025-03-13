use crate::tools::log_error_and_return;
use crate::tools::web::build_client;
use crate::uda::login::{check_credentials, get_authenticity_token};
use crate::web::authentication::UDA_AUTHENTICATION_COOKIE;
use crate::web::credentials::{CredentialsStorage, UdaCredentials};
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::time::Duration;
use std::sync::Mutex;
use uuid::Uuid;

/// Try and log a user onto UDA app.
/// If the login operation succeeds,
/// then a new UUID is created and credentials are stored with this UUID.
/// The UUID is returned to the caller through a private cookie, so that it is their new access token.
#[post("/uda/login", format = "application/json", data = "<credentials>")]
pub async fn login(
    credentials_storage: &State<Mutex<CredentialsStorage<UdaCredentials>>>,
    cookie_jar: &CookieJar<'_>,
    credentials: Json<UdaCredentials>,
) -> Result<(Status, ()), Status> {
    let client = build_client();
    if let Ok(client) = client {
        let url = credentials.uda_url();
        let login = credentials.login();
        let password = credentials.password();

        let authenticity_token = get_authenticity_token(&client, url)
            .await
            .map_err(log_error_and_return(Status::BadGateway))?;

        match check_credentials(&client, url, &authenticity_token, login, password).await {
            Ok(_) => {
                let mut mutex = credentials_storage
                    .lock()
                    .map_err(log_error_and_return(Status::InternalServerError))?;
                let uuid = Uuid::new_v4().to_string();
                let cookie = Cookie::build((UDA_AUTHENTICATION_COOKIE.to_owned(), uuid.clone()))
                    .max_age(Duration::days(365))
                    .build();
                cookie_jar.add_private(cookie);
                (*mutex).store(uuid.clone(), credentials.into_inner());
                Ok((Status::Ok, ()))
            }
            Err(error) => log_error_and_return(Err(Status::Unauthorized))(error),
        }
    } else {
        let error = client.unwrap_err();
        log_error_and_return(Err(Status::InternalServerError))(error)
    }
}
