use crate::database::dao::last_update::UpdatableElement::UdaInstances;
use crate::database::dao::last_update::get_last_update;
use crate::error::ApplicationError;
use crate::error::ApplicationError::Web;
use crate::tools::web::build_client;
use crate::tools::{log_error, log_error_and_return};
use crate::uda::authentication::AUTHENTICATION_COOKIE;
use crate::uda::configuration::Configuration;
use crate::uda::confirm_member::confirm_member;
use crate::uda::credentials::UdaCredentials;
use crate::uda::instances::retrieve_uda_instances;
use crate::web::credentials_storage::CredentialsStorage;
use crate::web::error::WebError::LackOfPermissions;
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use dto::uda_instance::InstancesList;
use r2d2::Pool;
use reqwest::Client;
use rocket::State;
use rocket::form::validate::Contains;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::{Json, Value, json};
use rocket::time::Duration;
use std::sync::Mutex;
use uda_connector::error::UdaError::ConnectionFailed;
use uda_connector::login::authenticate_into_uda;
use uda_connector::retrieve_members::retrieve_members;
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
    let cookie = Cookie::build((AUTHENTICATION_COOKIE.to_owned(), uuid.clone()))
        .max_age(Duration::days(365))
        .build();
    cookie_jar.add_private(cookie);
    (*mutex).store(uuid.clone(), credentials.into_inner());
    Ok(Status::Ok)
}

/// Retrieve all members from UDA's organisation membership page if authorized.
#[get("/uda/retrieve")]
pub async fn retrieve_members_to_check(credentials: UdaCredentials) -> Result<String, Status> {
    let client = build_client().map_err(log_error_and_return(Status::InternalServerError))?;
    authenticate(&client, &credentials).await?;
    let url = credentials.uda_url();
    match retrieve_members(&client, url).await {
        Ok(members) => Ok(json!(members).to_string()),
        Err(uda_connector::error::UdaError::LackOfPermissions) => Err(Status::Unauthorized),
        Err(_) => Err(Status::BadGateway),
    }
}

/// Confirm members on UDA if authorized.
/// Return a JSON containing members ids which have been marked as confirmed and whose confirmation has failed:
/// ```json
/// {
///     "ok": [id_1, id_2, ...],
///     "nok": [id_3, ...]
/// }
/// ```
#[post("/uda/confirm", format = "application/json", data = "<members_ids>")]
pub async fn confirm_members(
    members_ids: Json<Vec<u16>>,
    credentials: UdaCredentials,
) -> (Status, Value) {
    let members_ids = members_ids.into_inner();
    let client = match build_client() {
        Ok(client) => client,
        Err(error) => {
            log_error(error);
            return (
                Status::InternalServerError,
                json!({"ok": Vec::<u16>::new(), "nok":members_ids}),
            );
        }
    };

    if let Err(status) = authenticate(&client, &credentials).await {
        return (status, json!({"ok": Vec::<u16>::new(), "nok":members_ids}));
    };
    let url = credentials.uda_url();

    let mut not_marked_ids = vec![];
    let mut errors = vec![];
    for id in &members_ids {
        let result = confirm_member(&client, url, *id).await;
        if let Err(error) = result {
            debug!(
                "Member has not been confirmed. [member_id: {id}, error: {:?}]",
                error
            );
            not_marked_ids.push(*id);
            errors.push(error);
        }
    }

    let marked_ids: Vec<u16> = members_ids
        .iter()
        .filter(|id| !not_marked_ids.contains(**id))
        .copied()
        .collect();

    (
        if not_marked_ids.is_empty() {
            Status::Ok
        } else {
            from_vec_of_errors_to_status(&errors)
        },
        json!({"ok": marked_ids, "nok":not_marked_ids}),
    )
}

/// Retrieve and return a list of all existing UDA instances, alongside with the last update date
/// - i.e. the date this endpoint is called.
#[get("/uda/instances")]
pub async fn list_instances(
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
    configuration: &State<Configuration>,
) -> Result<Value, Status> {
    let client = build_client().map_err(log_error_and_return(Status::InternalServerError))?;
    let instances = retrieve_uda_instances(pool, &client, configuration.inner())
        .await
        .map_err(log_error_and_return(Status::BadGateway))?;
    let mut connection = pool
        .get()
        .map_err(log_error_and_return(Status::InternalServerError))?;
    let last_updated = get_last_update(&mut connection, &UdaInstances)
        .map_err(log_error_and_return(Status::InternalServerError))?
        .map(|naive_date_time| naive_date_time.date());

    Ok(json!(InstancesList::new(instances, last_updated)))
}

async fn authenticate(client: &Client, credentials: &UdaCredentials) -> Result<(), Status> {
    let url = credentials.uda_url();
    let login = credentials.login();
    let password = credentials.password();

    let authentication_result = authenticate_into_uda(client, url, login, password).await;
    if let Err(error) = authentication_result {
        match error {
            ConnectionFailed => Err(Status::BadGateway),
            _ => Err(Status::Unauthorized),
        }
    } else {
        Ok(())
    }
}

fn from_vec_of_errors_to_status(errors: &[ApplicationError]) -> Status {
    if errors
        .iter()
        .any(|error| matches!(*error, Web(LackOfPermissions)))
    {
        Status::Unauthorized
    } else {
        Status::BadGateway
    }
}

#[cfg(test)]
mod tests {
    mod login {
        use crate::uda::authentication::AUTHENTICATION_COOKIE;
        use crate::uda::credentials::UdaCredentials;
        use crate::web::api::uda_controller::login;
        use crate::web::credentials_storage::CredentialsStorage;
        use rocket::http::hyper::header::CONTENT_TYPE;
        use rocket::http::{ContentType, Header, Status};
        use rocket::local::asynchronous::Client;
        use rocket::serde::json::json;
        use std::sync::Mutex;
        use uda_connector::login::setup_authentication;
        use wiremock::matchers::{body_string, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            setup_authentication(&mock_server).await;

            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                "login".to_owned(),
                "password".to_owned(),
            )
            .into();
            let credentials_storage_mutex =
                Mutex::new(CredentialsStorage::<UdaCredentials>::default());

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
                    .get_private(AUTHENTICATION_COOKIE)
                    .is_some()
            );
        }

        #[async_test]
        async fn fail_when_bad_gateway() {
            let mock_server = MockServer::start().await;

            let body = "<html><body>Where do you think you are, son?</body></html>".to_string();
            Mock::given(method("GET"))
                .and(path("/en/users/sign_in"))
                .respond_with(ResponseTemplate::new(200).set_body_string(&body))
                .mount(&mock_server)
                .await;

            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                "login".to_owned(),
                "password".to_owned(),
            )
            .into();
            let credentials_storage_mutex =
                Mutex::new(CredentialsStorage::<UdaCredentials>::default());

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
                    .get_private(AUTHENTICATION_COOKIE)
                    .is_none()
            );
        }

        #[async_test]
        async fn fail_when_unauthorized() {
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

            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                "login".to_owned(),
                "password".to_owned(),
            )
            .into();
            let credentials_storage_mutex =
                Mutex::new(CredentialsStorage::<UdaCredentials>::default());

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
                    .get_private(AUTHENTICATION_COOKIE)
                    .is_none()
            );
        }
    }

    mod retrieve_members_to_check {
        use crate::uda::authentication::AUTHENTICATION_COOKIE;
        use crate::uda::credentials::UdaCredentials;
        use crate::web::api::uda_controller::retrieve_members_to_check;
        use crate::web::credentials_storage::CredentialsStorage;
        use dto::uda_member::UdaMember;
        use rocket::http::Status;
        use rocket::local::asynchronous::Client;
        use std::sync::Mutex;
        use uda_connector::login::setup_authentication;
        use uda_connector::retrieve_members::setup_member_retrieval;
        use wiremock::MockServer;

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            let credentials = setup_authentication(&mock_server).await;
            let expected_result = setup_member_retrieval(&mock_server).await;

            let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
            let mut credentials_storage = CredentialsStorage::<UdaCredentials>::default();
            credentials_storage.store(uuid.to_string(), credentials.into());
            let credentials_storage_mutex = Mutex::new(credentials_storage);

            let rocket = rocket::build()
                .manage(credentials_storage_mutex)
                .mount("/", routes![retrieve_members_to_check]);

            let client = Client::tracked(rocket).await.unwrap();
            let request = client
                .get("/uda/retrieve")
                .cookie((AUTHENTICATION_COOKIE, uuid));

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());
            let members: Vec<UdaMember> = response.into_json().await.unwrap();
            assert_eq!(expected_result, members);
        }

        #[async_test]
        async fn fail_when_unauthorized() {
            let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
            let credentials_storage_mutex =
                Mutex::new(CredentialsStorage::<UdaCredentials>::default());

            let rocket = rocket::build()
                .manage(credentials_storage_mutex)
                .mount("/", routes![retrieve_members_to_check]);

            let client = Client::tracked(rocket).await.unwrap();
            let request = client
                .get("/uda/retrieve")
                .cookie((AUTHENTICATION_COOKIE, uuid));

            let response = request.dispatch().await;
            assert_eq!(Status::Unauthorized, response.status());
        }

        #[async_test]
        async fn fail_when_bad_gateway() {
            let mock_server = MockServer::start().await;
            let credentials = setup_authentication(&mock_server).await;
            let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
            let mut credentials_storage = CredentialsStorage::<UdaCredentials>::default();
            credentials_storage.store(uuid.to_string(), credentials.into());
            let credentials_storage_mutex = Mutex::new(credentials_storage);

            let rocket = rocket::build()
                .manage(credentials_storage_mutex)
                .mount("/", routes![retrieve_members_to_check]);

            let client = Client::tracked(rocket).await.unwrap();
            let request = client
                .get("/uda/retrieve")
                .cookie((AUTHENTICATION_COOKIE, uuid));

            let response = request.dispatch().await;
            assert_eq!(Status::BadGateway, response.status());
        }
    }

    mod confirm_members {
        use crate::uda::confirm_member::tests::{setup_confirm_member, setup_csrf_token};
        use crate::uda::credentials::UdaCredentials;
        use crate::web::api::uda_controller::confirm_members;
        use rocket::http::Status;
        use rocket::serde::json::Json;
        use std::collections::HashMap;
        use uda_connector::login::{setup_authentication, setup_authenticity_token};
        use wiremock::MockServer;

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            let credentials = setup_authentication(&mock_server).await;
            let csrf_token = setup_csrf_token(&mock_server).await;
            setup_confirm_member(&mock_server, &csrf_token, 1).await;
            setup_confirm_member(&mock_server, &csrf_token, 2).await;
            setup_confirm_member(&mock_server, &csrf_token, 3).await;

            let (status, value) =
                confirm_members(Json::from(vec![1_u16, 2_u16, 3_u16]), credentials.into()).await;

            assert_eq!(Status::Ok, status);
            let result: HashMap<String, Vec<u16>> = rocket::serde::json::from_value(value).unwrap();
            assert_eq!(&vec![1_u16, 2_u16, 3_u16], result.get("ok").unwrap());
            assert_eq!(&Vec::<u16>::new(), result.get("nok").unwrap());
        }

        #[async_test]
        async fn fail_to_confirm_some_members() {
            let mock_server = MockServer::start().await;
            let credentials = setup_authentication(&mock_server).await;
            let csrf_token = setup_csrf_token(&mock_server).await;
            setup_confirm_member(&mock_server, &csrf_token, 1).await;

            let (status, value) =
                confirm_members(Json::from(vec![1, 2, 3]), credentials.into()).await;

            assert_eq!(Status::Unauthorized, status);
            let result: HashMap<String, Vec<u16>> = rocket::serde::json::from_value(value).unwrap();
            assert_eq!(&vec![1], result.get("ok").unwrap());
            assert_eq!(&vec![2, 3], result.get("nok").unwrap());
        }

        #[async_test]
        async fn fail_when_no_authentication() {
            let mock_server = MockServer::start().await;
            setup_authenticity_token(&mock_server).await;

            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                "login".to_owned(),
                "password".to_owned(),
            )
            .into();

            let (status, value) = confirm_members(Json::from(vec![1, 2, 3]), credentials).await;

            assert_eq!(Status::Unauthorized, status);
            let result: HashMap<String, Vec<u16>> = rocket::serde::json::from_value(value).unwrap();
            assert_eq!(&Vec::<u16>::new(), result.get("ok").unwrap());
            assert_eq!(&vec![1, 2, 3], result.get("nok").unwrap());
        }
    }

    mod list_instances {
        use crate::database::with_temp_database;
        use crate::uda::configuration::Configuration;
        use crate::web::api::uda_controller::list_instances;
        use diesel::SqliteConnection;
        use diesel::r2d2::ConnectionManager;
        use dto::uda_instance::Instance;
        use dto::uda_instance::InstancesList;
        use r2d2::Pool;
        use rocket::http::Status;
        use rocket::local::asynchronous::Client;
        use rocket::tokio::runtime::Runtime;
        use uda_connector::instances::{BODY, get_expected_instances};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test]
        fn success() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                Mock::given(method("GET"))
                    .and(path("tenants"))
                    .respond_with(ResponseTemplate::new(200).set_body_string(BODY))
                    .mount(&mock_server)
                    .await;
                let mut connection = pool.get().unwrap();

                let configuration =
                    Configuration::new(format!("{}/tenants?locale=en", mock_server.uri()));
                let rocket = rocket::build()
                    .manage(pool)
                    .manage(configuration)
                    .mount("/", routes![list_instances]);

                let client = Client::tracked(rocket).await.unwrap();
                let request = client.get("/uda/instances");

                let instances_list: InstancesList =
                    request.dispatch().await.into_json().await.unwrap();
                assert_eq!(&get_expected_instances(), instances_list.instances());
                let instances =
                    crate::database::dao::uda_instance::retrieve_all(&mut connection).unwrap();
                assert_eq!(&instances, instances_list.instances());
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_bad_gateway() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                Mock::given(method("GET"))
                    .and(path("tenants"))
                    .respond_with(ResponseTemplate::new(502))
                    .mount(&mock_server)
                    .await;

                let mut connection = pool.get().unwrap();
                let configuration =
                    Configuration::new(format!("{}/tenants?locale=en", mock_server.uri()));
                let rocket = rocket::build()
                    .manage(pool)
                    .manage(configuration)
                    .mount("/", routes![list_instances]);

                let client = Client::tracked(rocket).await.unwrap();
                let request = client.get("/uda/instances");

                let status = request.dispatch().await.status();
                assert_eq!(Status::BadGateway, status);
                let instances =
                    crate::database::dao::uda_instance::retrieve_all(&mut connection).unwrap();
                assert_eq!(Vec::<Instance>::new(), instances);
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }

    mod authenticate {
        use crate::uda::credentials::UdaCredentials;
        use crate::web::api::uda_controller::authenticate;
        use rocket::http::Status;
        use uda_connector::login::{setup_authentication, setup_authenticity_token};
        use wiremock::matchers::{body_string, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            let credentials = setup_authentication(&mock_server).await;

            let client = reqwest::Client::new();
            authenticate(&client, &credentials.into()).await.unwrap();
        }

        #[async_test]
        async fn fail_when_bad_gateway() {
            let login = "login";
            let password = "password";

            let mock_server = MockServer::start().await;
            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                login.to_owned(),
                password.to_owned(),
            )
            .into();

            let client = reqwest::Client::new();
            assert_eq!(
                Status::BadGateway,
                authenticate(&client, &credentials).await.unwrap_err()
            );
        }

        #[async_test]
        async fn fail_when_unauthorized() {
            let login = "login";
            let password = "password";
            let mock_server = MockServer::start().await;
            let credentials: UdaCredentials = uda_connector::credentials::UdaCredentials::new(
                mock_server.uri(),
                login.to_owned(),
                password.to_owned(),
            )
            .into();
            let authenticity_token = setup_authenticity_token(&mock_server).await;
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
    }
}
