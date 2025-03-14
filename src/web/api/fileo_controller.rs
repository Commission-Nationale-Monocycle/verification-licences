use crate::member::config::MembershipsProviderConfig;
use crate::member::download::{download_memberships_list, login_to_fileo};
use crate::member::import_from_file::{clean_old_files, import_from_file};
use crate::tools::web::build_client;
use crate::tools::{log_error_and_return, log_message, log_message_and_return};
use crate::web::api::members_state::MembersState;
use crate::web::authentication::FILEO_AUTHENTICATION_COOKIE;
use crate::web::credentials::{CredentialsStorage, FileoCredentials};
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::time::Duration;
use serde_json::json;
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
    let client = build_client();
    if let Ok(client) = client {
        let host = memberships_provider_config.inner().host();
        let credentials = credentials.into_inner();
        match login_to_fileo(&client, host, &credentials).await {
            Ok(_) => {
                let mut mutex = credentials_storage
                    .lock()
                    .map_err(log_error_and_return(Status::InternalServerError))?;
                let uuid = Uuid::new_v4().to_string();
                let cookie = Cookie::build((FILEO_AUTHENTICATION_COOKIE.to_owned(), uuid.clone()))
                    .max_age(Duration::days(365))
                    .build();
                cookie_jar.add_private(cookie);
                (*mutex).store(uuid.clone(), credentials);
                Ok((Status::Ok, ()))
            }
            Err(error) => log_error_and_return(Err(Status::Unauthorized))(error),
        }
    } else {
        let error = client.unwrap_err();
        log_error_and_return(Err(Status::InternalServerError))(error)
    }
}

/// Download memberships csv file from remote provided in config,
/// write said file into filesystem
/// and load it into memory.
/// Finally, clean all old memberships files.
#[get("/fileo/memberships", format = "text/plain-text")]
pub async fn download_memberships(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    members_state: &State<Mutex<MembersState>>,
    credentials: FileoCredentials,
) -> Result<String, Status> {
    let file_details = download_memberships_list(memberships_provider_config, &credentials)
        .await
        .map_err(log_message_and_return(
            "Can't download memberships list",
            Status::InternalServerError,
        ))?;

    let members = import_from_file(file_details.filepath()).map_err(log_message_and_return(
        "Error while reading memberships file.",
        Status::InternalServerError,
    ))?;
    let mut members_state = members_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        Status::InternalServerError,
    ))?;

    let file_update_date = *file_details.update_date();
    clean_old_files(memberships_provider_config.folder(), &file_update_date)
        .map_err(log_message("Couldn't clean old files."))
        .ok();
    members_state.set_file_details(file_details);
    members_state.set_members(members);
    Ok(json!(members_state.members()).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::member::config::MembershipsProviderConfig;
    use crate::member::members::Members;
    use crate::tools::test::tests::temp_dir;
    use crate::web::api::members_state::MembersState;
    use dto::membership::tests::{MEMBERSHIP_NUMBER, get_expected_member, get_member_as_csv};
    use encoding::all::ISO_8859_1;
    use encoding::{EncoderTrap, Encoding};
    use regex::Regex;
    use reqwest::header::CONTENT_TYPE;
    use rocket::State;
    use rocket::http::{ContentType, Header};
    use rocket::local::asynchronous::Client;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_memberships_provider_test_config(uri: &str) -> MembershipsProviderConfig {
        let temp_dir = temp_dir();
        MembershipsProviderConfig::new(
            uri.to_owned(),
            Regex::new(&format!("{}/download\\.csv", uri)).unwrap(),
            temp_dir.into_os_string(),
        )
    }

    fn create_member_state_mutex() -> Mutex<MembersState> {
        Mutex::new(MembersState::default())
    }

    async fn setup_login(mock_server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(200))
            .mount(mock_server)
            .await;
    }

    // region login
    #[async_test]
    async fn should_login() {
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
            .manage(create_member_state_mutex())
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
                .get_private(FILEO_AUTHENTICATION_COOKIE)
                .is_some()
        );
    }

    #[async_test]
    async fn should_not_login_when_fileo_login_failed() {
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
            .manage(create_member_state_mutex())
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
        assert_eq!(Status::Unauthorized, response.status());
        assert!(
            response
                .cookies()
                .get_private(FILEO_AUTHENTICATION_COOKIE)
                .is_none()
        );
    }
    // endregion

    // region download_members
    #[async_test]
    async fn should_download_members() {
        let mock_server = MockServer::start().await;

        let config = create_memberships_provider_test_config(&mock_server.uri());
        let old_file_path = PathBuf::from(config.folder()).join("memberships-1980-01-01.csv");
        let download_filename = "download.csv";
        let download_link = format!("{}/{download_filename}", mock_server.uri());

        fs::write(&old_file_path, "").unwrap();
        assert!(fs::exists(&old_file_path).ok().unwrap());

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
        let member_as_csv = get_member_as_csv();
        let member_as_csv = ISO_8859_1
            .encode(&member_as_csv, EncoderTrap::Strict)
            .unwrap();
        let message_in_latin1: &[u8] = &member_as_csv;
        Mock::given(method("GET"))
            .and(path(format!("/{download_filename}").to_owned()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"))
            .mount(&mock_server)
            .await;

        let config_state = State::from(&config);
        let members_state_mutex = Mutex::new(MembersState::new(None, Members::default()));
        let members_state = State::from(&members_state_mutex);
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());

        let result = download_memberships(config_state, members_state, credentials)
            .await
            .unwrap();
        let members: Members = serde_json::from_str(&result).unwrap();
        assert_eq!(
            &get_expected_member(),
            members
                .get(MEMBERSHIP_NUMBER)
                .unwrap()
                .iter()
                .find(|_| true)
                .unwrap()
        );
        assert!(
            !fs::exists(&old_file_path).ok().unwrap(),
            "Old file should have been cleaned."
        );
    }

    #[async_test]
    async fn should_not_download_members_when_error() {
        let mock_server = MockServer::start().await;

        let config = create_memberships_provider_test_config(&mock_server.uri());

        let config_state = State::from(&config);
        let members_state_mutex = Mutex::new(MembersState::new(None, Members::default()));
        let members_state = State::from(&members_state_mutex);
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());

        let result = download_memberships(config_state, members_state, credentials).await;
        assert_eq!(Status::InternalServerError, result.unwrap_err());
    }
    // endregion
}
