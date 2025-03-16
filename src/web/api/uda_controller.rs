use crate::tools::error::Error;
use crate::tools::error::Error::LackOfPermissions;
use crate::tools::log_error_and_return;
use crate::tools::web::build_client;
use crate::uda::login::authenticate_into_uda;
use crate::uda::retrieve_members::retrieve_members;
use crate::web::authentication::UDA_AUTHENTICATION_COOKIE;
use crate::web::credentials::{CredentialsStorage, UdaCredentials};
use reqwest::Client;
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::time::Duration;
use serde_json::json;
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
) -> Result<Status, Status> {
    let client = build_client().map_err(log_error_and_return(Status::InternalServerError))?;
    authenticate(&client, &credentials).await?;
    let mut mutex = credentials_storage
        .lock()
        .map_err(log_error_and_return(Status::InternalServerError))?;
    let uuid = Uuid::new_v4().to_string();
    let cookie = Cookie::build((UDA_AUTHENTICATION_COOKIE.to_owned(), uuid.clone()))
        .max_age(Duration::days(365))
        .build();
    cookie_jar.add_private(cookie);
    (*mutex).store(uuid.clone(), credentials.into_inner());
    Ok(Status::Ok)
}

#[get("/uda/retrieve")]
pub async fn retrieve_members_to_check(credentials: UdaCredentials) -> Result<String, Status> {
    let client = build_client().map_err(log_error_and_return(Status::InternalServerError))?;
    authenticate(&client, &credentials).await?;
    let url = credentials.uda_url();
    match retrieve_members(&client, url).await {
        Ok(members_to_check) => Ok(json!(members_to_check).to_string()),
        Err(LackOfPermissions) => Err(Status::Unauthorized),
        Err(_) => Err(Status::BadGateway),
    }
}

async fn authenticate(client: &Client, credentials: &UdaCredentials) -> Result<(), Status> {
    let url = credentials.uda_url();
    let login = credentials.login();
    let password = credentials.password();

    let authentication_result = authenticate_into_uda(client, url, login, password).await;
    if let Err(error) = authentication_result {
        match error {
            Error::ConnectionFailed => Err(Status::BadGateway),
            _ => Err(Status::Unauthorized),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uda::login::tests::setup_authentication;
    use crate::uda::retrieve_members::tests::setup_members_to_check_retrieval;
    use dto::member_to_check::MemberToCheck;
    use reqwest::header::CONTENT_TYPE;
    use rocket::http::{ContentType, Header};
    use rocket::local::asynchronous::Client;
    use serde_json::json;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // region login
    #[async_test]
    async fn should_login() {
        let mock_server = MockServer::start().await;
        setup_authentication(&mock_server).await;

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
    async fn should_fail_to_login_when_unauthorized() {
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
    // endregion

    // region retrieve_members_to_check
    #[async_test]
    async fn should_retrieve_members_to_check() {
        let mock_server = MockServer::start().await;
        let credentials = setup_authentication(&mock_server).await;
        let expected_result = setup_members_to_check_retrieval(&mock_server).await;

        let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
        let mut credentials_storage = CredentialsStorage::<UdaCredentials>::default();
        credentials_storage.store(uuid.to_string(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![retrieve_members_to_check]);

        let client = Client::tracked(rocket).await.unwrap();
        let request = client
            .get("/uda/retrieve")
            .cookie((UDA_AUTHENTICATION_COOKIE, uuid));

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
        let members_to_check: Vec<MemberToCheck> = response.into_json().await.unwrap();
        assert_eq!(expected_result, members_to_check);
    }

    #[async_test]
    async fn should_fail_to_retrieve_members_to_check_when_unauthorized() {
        let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
        let credentials_storage_mutex = Mutex::new(CredentialsStorage::<UdaCredentials>::default());

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![retrieve_members_to_check]);

        let client = Client::tracked(rocket).await.unwrap();
        let request = client
            .get("/uda/retrieve")
            .cookie((UDA_AUTHENTICATION_COOKIE, uuid));

        let response = request.dispatch().await;
        assert_eq!(Status::Unauthorized, response.status());
    }

    #[async_test]
    async fn should_fail_to_retrieve_members_to_check_when_bad_gateway() {
        let mock_server = MockServer::start().await;
        let credentials = setup_authentication(&mock_server).await;
        let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
        let mut credentials_storage = CredentialsStorage::<UdaCredentials>::default();
        credentials_storage.store(uuid.to_string(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build()
            .manage(credentials_storage_mutex)
            .mount("/", routes![retrieve_members_to_check]);

        let client = Client::tracked(rocket).await.unwrap();
        let request = client
            .get("/uda/retrieve")
            .cookie((UDA_AUTHENTICATION_COOKIE, uuid));

        let response = request.dispatch().await;
        assert_eq!(Status::BadGateway, response.status());
    }

    // endregion

    // region authenticate
    #[async_test]
    async fn should_authenticate() {
        let mock_server = MockServer::start().await;
        let credentials = setup_authentication(&mock_server).await;

        let client = reqwest::Client::new();
        authenticate(&client, &credentials).await.unwrap();
    }

    #[async_test]
    async fn should_fail_to_authenticate_when_bad_gateway() {
        let login = "login";
        let password = "password";

        let mock_server = MockServer::start().await;
        let credentials =
            UdaCredentials::new(mock_server.uri(), login.to_owned(), password.to_owned());

        let client = reqwest::Client::new();
        assert_eq!(
            Status::BadGateway,
            authenticate(&client, &credentials).await.unwrap_err()
        );
    }

    #[async_test]
    async fn should_fail_to_authenticate_when_unauthorized() {
        let login = "login";
        let password = "password";
        let mock_server = MockServer::start().await;
        let credentials =
            UdaCredentials::new(mock_server.uri(), login.to_owned(), password.to_owned());
        let authenticity_token =
            crate::uda::login::tests::setup_authenticity_token(&mock_server).await;
        let params = format!(
            "user%5Bemail%5D={login}&user%5Bpassword%5D={password}&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<html><body>Invalid User Account Email or password</body></html>",
            ))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        assert_eq!(
            Status::Unauthorized,
            authenticate(&client, &credentials).await.unwrap_err()
        );
    }

    // endregion
}
