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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::CONTENT_TYPE;
    use rocket::http::{ContentType, Header};
    use rocket::local::asynchronous::Client;
    use serde_json::json;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[async_test]
    async fn should_login() {
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{authenticity_token}"></body></html>"#
        );
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string("Signed in successfully"))
            .mount(&mock_server)
            .await;

        let credentials =
            UdaCredentials::new(mock_server.uri(), "login".to_owned(), "password".to_owned());
        let credentials_storage_mutex = Mutex::new(CredentialsStorage::<UdaCredentials>::default());

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![login]);
        let client = Client::tracked(rocket).await.unwrap();
        let credentials_as_json = json!(credentials).to_string();
        let request = client
            .post("/uda/login")
            .body(credentials_as_json.as_bytes())
            .header(Header::new(
                CONTENT_TYPE.to_string(),
                ContentType::JSON.to_string(),
            ));

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
        assert!(
            response
                .cookies()
                .get_private(UDA_AUTHENTICATION_COOKIE)
                .is_some()
        );
    }

    #[async_test]
    async fn should_fail_to_login_when_bad_gateway() {
        let mock_server = MockServer::start().await;

        let body = "<html><body>Where do you think you are, son?</body></html>".to_string();
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;

        let credentials =
            UdaCredentials::new(mock_server.uri(), "login".to_owned(), "password".to_owned());
        let credentials_storage_mutex = Mutex::new(CredentialsStorage::<UdaCredentials>::default());

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![login]);
        let client = Client::tracked(rocket).await.unwrap();
        let credentials_as_json = json!(credentials).to_string();
        let request = client
            .post("/uda/login")
            .body(credentials_as_json.as_bytes())
            .header(Header::new(
                CONTENT_TYPE.to_string(),
                ContentType::JSON.to_string(),
            ));

        let response = request.dispatch().await;
        assert_eq!(Status::BadGateway, response.status());
        assert!(
            response
                .cookies()
                .get_private(UDA_AUTHENTICATION_COOKIE)
                .is_none()
        );
    }

    #[async_test]
    async fn should_fail_to_login_when_lsbad_gateway() {
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{authenticity_token}"></body></html>"#
        );
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;

        let params = format!(
            "user%5Bemail%5D=wrong_login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string("Signed in successfully"))
            .mount(&mock_server)
            .await;

        let credentials =
            UdaCredentials::new(mock_server.uri(), "login".to_owned(), "password".to_owned());
        let credentials_storage_mutex = Mutex::new(CredentialsStorage::<UdaCredentials>::default());

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![login]);
        let client = Client::tracked(rocket).await.unwrap();
        let credentials_as_json = json!(credentials).to_string();
        let request = client
            .post("/uda/login")
            .body(credentials_as_json.as_bytes())
            .header(Header::new(
                CONTENT_TYPE.to_string(),
                ContentType::JSON.to_string(),
            ));

        let response = request.dispatch().await;
        assert_eq!(Status::Unauthorized, response.status());
        assert!(
            response
                .cookies()
                .get_private(UDA_AUTHENTICATION_COOKIE)
                .is_none()
        );
    }
}
