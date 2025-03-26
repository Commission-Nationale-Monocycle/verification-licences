use crate::fileo::authentication::AUTHENTICATION_COOKIE;
use crate::fileo::credentials::FileoCredentials;
use crate::fileo::download::{download_memberships_list, login_to_fileo};
use crate::membership::config::MembershipsProviderConfig;
use crate::membership::import_from_file::{clean_old_files, import_from_file};
use crate::tools::web::build_client;
use crate::tools::{log_error_and_return, log_message, log_message_and_return};
use crate::web::api::memberships_state::MembershipsState;
use crate::web::credentials_storage::CredentialsStorage;
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::{Json, json};
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
        Err(error) => log_error_and_return(Err(Status::Unauthorized))(error),
    }
}

/// Download memberships csv file from remote provided in config,
/// write said file into filesystem
/// and load it into memory.
/// Finally, clean all old memberships files.
#[get("/fileo/memberships", format = "text/plain-text")]
pub async fn download_memberships(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    memberships_state: &State<Mutex<MembershipsState>>,
    credentials: FileoCredentials,
) -> Result<Status, Status> {
    let file_details = download_memberships_list(memberships_provider_config, &credentials)
        .await
        .map_err(log_message_and_return(
            "Can't download memberships list",
            Status::InternalServerError,
        ))?;

    let memberships = import_from_file(file_details.filepath()).map_err(log_message_and_return(
        "Error while reading memberships file.",
        Status::InternalServerError,
    ))?;
    let mut memberships_state = memberships_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        Status::InternalServerError,
    ))?;

    let file_update_date = *file_details.update_date();
    clean_old_files(memberships_provider_config.folder(), &file_update_date)
        .map_err(log_message("Couldn't clean old files."))
        .ok();
    memberships_state.set_file_details(file_details);
    memberships_state.set_memberships(memberships);

    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::membership::config::MembershipsProviderConfig;
    use crate::membership::indexed_memberships::IndexedMemberships;
    use crate::tools::test::tests::temp_dir;
    use crate::web::api::memberships_state::MembershipsState;
    use dto::membership::tests::{get_expected_membership, get_membership_as_csv};
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

    fn create_member_state_mutex() -> Mutex<MembershipsState> {
        Mutex::new(MembershipsState::default())
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
                .get_private(AUTHENTICATION_COOKIE)
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
                .get_private(AUTHENTICATION_COOKIE)
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
        let member_as_csv = get_membership_as_csv();
        let member_as_csv = ISO_8859_1
            .encode(&member_as_csv, EncoderTrap::Strict)
            .unwrap();
        let message_in_latin1: &[u8] = &member_as_csv;
        Mock::given(method("GET"))
            .and(path(format!("/{download_filename}").to_owned()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"))
            .mount(&mock_server)
            .await;

        let memberships_state_mutex =
            Mutex::new(MembershipsState::new(None, IndexedMemberships::default()));
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store(uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let rocket = rocket::build()
            .manage(config)
            .manage(memberships_state_mutex)
            .manage(credentials_storage_mutex)
            .mount("/", routes![download_memberships]);
        let client = Client::tracked(rocket).await.unwrap();

        let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);
        let request = client.get("/fileo/memberships").cookie(cookie);
        let response = request.dispatch().await;

        assert_eq!(Status::NoContent, response.status());

        let membership_state = client.rocket().state::<Mutex<MembershipsState>>().unwrap();
        let membership_state = membership_state.lock().unwrap();
        let members = membership_state.memberships();
        assert_eq!(&get_expected_membership(), members.first().unwrap());
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
        let memberships_state_mutex =
            Mutex::new(MembershipsState::new(None, IndexedMemberships::default()));
        let memberships_state = State::from(&memberships_state_mutex);
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());

        let result = download_memberships(config_state, memberships_state, credentials).await;
        assert_eq!(Status::InternalServerError, result.unwrap_err());
    }
    // endregion
}
