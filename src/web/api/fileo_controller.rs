use crate::database::dao::membership::replace_memberships;
use crate::error::ApplicationError;
use crate::fileo::authentication::AUTHENTICATION_COOKIE;
use crate::fileo::credentials::FileoCredentials;
use crate::fileo::download::{download_memberships_list, login_to_fileo};
use crate::membership::config::MembershipsProviderConfig;
use crate::tools::web::build_client;
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::web::credentials_storage::CredentialsStorage;
use crate::web::error::WebError;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::time::Duration;
use std::sync::Mutex;
use uuid::Uuid;

/// Try and log a user onto Fileo app.
/// If the login operation succeeds,
/// then a new UUID is created and credentials are stored with this UUID.
/// The UUID is returned to the caller through a private cookie, so that it is their new access token.
#[post("/fileo/login", format = "application/json", data = "<credentials>")]
pub async fn login(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    credentials_storage: &State<Mutex<CredentialsStorage<FileoCredentials>>>,
    cookie_jar: &CookieJar<'_>,
    credentials: Json<FileoCredentials>,
) -> Result<(Status, ()), Status> {
    let client = build_client().map_err(log_error_and_return(Status::InternalServerError))?;
    let host = memberships_provider_config.inner().host();
    let credentials = credentials.into_inner();
    match login_to_fileo(&client, host, &credentials).await {
        Ok(_) => {
            let mut mutex = credentials_storage
                .lock()
                .map_err(log_error_and_return(Status::InternalServerError))?;
            let uuid = Uuid::new_v4().to_string();
            let cookie = Cookie::build((AUTHENTICATION_COOKIE.to_owned(), uuid.clone()))
                .max_age(Duration::days(365))
                .build();
            cookie_jar.add_private(cookie);
            (*mutex).store(uuid.clone(), credentials);
            Ok((Status::Ok, ()))
        }
        Err(ApplicationError::Web(WebError::LackOfPermissions)) => Err(Status::Forbidden),
        Err(ApplicationError::Web(WebError::WrongCredentials)) => Err(Status::Unauthorized),
        Err(_) => Err(Status::BadGateway),
    }
}

/// Download memberships csv file from remote provided in config,
/// write said file into filesystem
/// and load it into memory.
/// Finally, clean all old memberships files.
#[get("/fileo/memberships", format = "text/plain-text")]
pub async fn download_memberships(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
    credentials: FileoCredentials,
) -> Result<Status, Status> {
    let memberships = download_memberships_list(memberships_provider_config, &credentials)
        .await
        .map_err(log_message_and_return(
            "Can't download memberships list",
            Status::InternalServerError,
        ))?;

    let mut connection = pool
        .get()
        .map_err(log_error_and_return(Status::InternalServerError))?;
    replace_memberships(&mut connection, &memberships)
        .map_err(log_error_and_return(Status::InternalServerError))?;

    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use crate::membership::config::MembershipsProviderConfig;
    use regex::Regex;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_memberships_provider_test_config(uri: &str) -> MembershipsProviderConfig {
        MembershipsProviderConfig::new(
            uri.to_owned(),
            Regex::new(&format!("{}/download\\.csv", uri)).unwrap(),
        )
    }

    async fn setup_login(mock_server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "Profil Commission thématique - UNSLL - Commission Nationale Monocycle",
            ))
            .mount(mock_server)
            .await;
    }

    mod login {
        use crate::database::with_temp_database;
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::web::api::fileo_controller::login;
        use crate::web::api::fileo_controller::tests::{
            create_memberships_provider_test_config, setup_login,
        };
        use crate::web::credentials_storage::CredentialsStorage;
        use diesel::SqliteConnection;
        use diesel::r2d2::{ConnectionManager, Pool};
        use reqwest::header::CONTENT_TYPE;
        use rocket::http::{ContentType, Header, Status};
        use rocket::local::asynchronous::Client;
        use rocket::serde::json::json;
        use rocket::tokio::runtime::Runtime;
        use std::sync::Mutex;
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test]
        fn success() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                setup_login(&mock_server).await;

                let config = create_memberships_provider_test_config(&mock_server.uri());

                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let credentials_storage_mutex =
                    Mutex::new(CredentialsStorage::<FileoCredentials>::default());

                let rocket = rocket::build()
                    .manage(config)
                    .manage(credentials_storage_mutex)
                    .manage(pool)
                    .mount("/", routes![login]);
                let client = Client::tracked(rocket).await.unwrap();
                let credentials_as_json = json!(credentials).to_string();
                let request = client
                    .post("/fileo/login")
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
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_unauthorized() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                Mock::given(method("POST"))
                    .and(path("/page.php"))
                    .and(body_string_contains("Action=connect_user"))
                    .respond_with(ResponseTemplate::new(401))
                    .mount(&mock_server)
                    .await;

                let config = create_memberships_provider_test_config(&mock_server.uri());

                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let credentials_storage_mutex =
                    Mutex::new(CredentialsStorage::<FileoCredentials>::default());

                let rocket = rocket::build()
                    .manage(config)
                    .manage(credentials_storage_mutex)
                    .manage(pool)
                    .mount("/", routes![login]);
                let client = Client::tracked(rocket).await.unwrap();
                let credentials_as_json = json!(credentials).to_string();
                let request = client
                    .post("/fileo/login")
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
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_forbidden() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                Mock::given(method("POST"))
                    .and(path("/page.php"))
                    .and(body_string_contains("Action=connect_user"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_string("Profil Club - The Best Unicycle Club Ever"),
                    )
                    .mount(&mock_server)
                    .await;

                let config = create_memberships_provider_test_config(&mock_server.uri());

                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let credentials_storage_mutex =
                    Mutex::new(CredentialsStorage::<FileoCredentials>::default());

                let rocket = rocket::build()
                    .manage(config)
                    .manage(credentials_storage_mutex)
                    .manage(pool)
                    .mount("/", routes![login]);
                let client = Client::tracked(rocket).await.unwrap();
                let credentials_as_json = json!(credentials).to_string();
                let request = client
                    .post("/fileo/login")
                    .body(credentials_as_json.as_bytes())
                    .header(Header::new(
                        CONTENT_TYPE.to_string(),
                        ContentType::JSON.to_string(),
                    ));

                let response = request.dispatch().await;
                assert_eq!(Status::Forbidden, response.status());
                assert!(
                    response
                        .cookies()
                        .get_private(AUTHENTICATION_COOKIE)
                        .is_none()
                );
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_bad_gateway() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;
                Mock::given(method("POST"))
                    .and(path("/page.php"))
                    .and(body_string_contains("Action=connect_user"))
                    .respond_with(ResponseTemplate::new(500))
                    .mount(&mock_server)
                    .await;

                let config = create_memberships_provider_test_config(&mock_server.uri());

                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let credentials_storage_mutex =
                    Mutex::new(CredentialsStorage::<FileoCredentials>::default());

                let rocket = rocket::build()
                    .manage(config)
                    .manage(credentials_storage_mutex)
                    .manage(pool)
                    .mount("/", routes![login]);
                let client = Client::tracked(rocket).await.unwrap();
                let credentials_as_json = json!(credentials).to_string();
                let request = client
                    .post("/fileo/login")
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
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }

    mod download_members {
        use crate::database::{dao, with_temp_database};
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::web::api::fileo_controller::download_memberships;
        use crate::web::api::fileo_controller::tests::{
            create_memberships_provider_test_config, setup_login,
        };
        use crate::web::credentials_storage::CredentialsStorage;
        use diesel::SqliteConnection;
        use diesel::r2d2::{ConnectionManager, Pool};
        use dto::membership::tests::{get_expected_membership, get_membership_as_csv};
        use encoding::all::ISO_8859_1;
        use encoding::{EncoderTrap, Encoding};
        use rocket::State;
        use rocket::http::{Cookie, Status};
        use rocket::local::asynchronous::Client;
        use rocket::tokio::runtime::Runtime;
        use std::sync::Mutex;
        use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test]
        fn should_download_members() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;

                let config = create_memberships_provider_test_config(&mock_server.uri());
                let download_filename = "download.csv";
                let download_link = format!("{}/{download_filename}", mock_server.uri());

                setup_login(&mock_server).await;
                Mock::given(method("POST"))
                    .and(path("/page.php"))
                    .and(query_param_contains(
                        "P",
                        "bo/extranet/adhesion/annuaire/index",
                    ))
                    .and(body_string_contains("Action=adherent_filtrer"))
                    .respond_with(ResponseTemplate::new(200))
                    .mount(&mock_server)
                    .await;
                Mock::given(method("POST"))
                    .and(path("/includer.php"))
                    .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
                    .respond_with(ResponseTemplate::new(200).set_body_raw(
                        format!("<p>Here is the download link: {download_link}</p>"),
                        "text/html",
                    ))
                    .mount(&mock_server)
                    .await;
                let member_as_csv = get_membership_as_csv();
                let member_as_csv = ISO_8859_1
                    .encode(&member_as_csv, EncoderTrap::Strict)
                    .unwrap();
                let message_in_latin1: &[u8] = &member_as_csv;
                Mock::given(method("GET"))
                    .and(path(format!("/{download_filename}").to_owned()))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"),
                    )
                    .mount(&mock_server)
                    .await;

                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let mut credentials_storage = CredentialsStorage::default();
                let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
                credentials_storage.store(uuid.clone(), credentials);
                let credentials_storage_mutex = Mutex::new(credentials_storage);

                let rocket = rocket::build()
                    .manage(config)
                    .manage(credentials_storage_mutex)
                    .manage(pool)
                    .mount("/", routes![download_memberships]);
                let client = Client::tracked(rocket).await.unwrap();

                let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);
                let request = client.get("/fileo/memberships").cookie(cookie);
                let response = request.dispatch().await;

                assert_eq!(Status::NoContent, response.status());

                let mut connection = client
                    .rocket()
                    .state::<Pool<ConnectionManager<SqliteConnection>>>()
                    .unwrap()
                    .get()
                    .unwrap();
                let membership = dao::membership::retrieve_memberships(&mut connection).unwrap();
                assert_eq!(&get_expected_membership(), membership.first().unwrap());
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn should_not_download_members_when_error() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;

                let config = create_memberships_provider_test_config(&mock_server.uri());

                let config_state = State::from(&config);
                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let pool_state = State::from(&pool);

                let result = download_memberships(config_state, pool_state, credentials).await;
                assert_eq!(Status::InternalServerError, result.unwrap_err());
            }

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }
}
