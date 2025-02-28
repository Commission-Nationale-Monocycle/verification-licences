use std::sync::Mutex;
use rocket::State;
use serde_json::json;
use crate::member::config::MembersProviderConfig;
use crate::member::download::download_members_list;
use crate::member::import_from_file::import_from_file;
use crate::web::api::members_state::MembersState;
use crate::tools::log_message_and_return;

/// Download members csv file from remote provided in config,
/// write said file into filesystem
/// and load it into memory.
#[get("/members")]
pub async fn download_members(
    members_provider_config: &State<MembersProviderConfig>,
    members_state: &State<Mutex<MembersState>>,
) -> Result<String, String> {
    let file_details = download_members_list(members_provider_config)
        .await
        .map_err(log_message_and_return("Can't download members list", "Can't download members list"))?;

    let members = import_from_file(file_details.filepath()).map_err(log_message_and_return("Error while reading members file.", "Error while reading members file."))?;
    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while updating members."))?;

    members_state.set_file_details(file_details);
    members_state.set_members(members);
    Ok(json!(members_state.members()).to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use encoding::all::ISO_8859_1;
    use encoding::{EncoderTrap, Encoding};
    use regex::Regex;
    use rocket::futures::executor::block_on;
    use rocket::State;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
    use crate::member::Members;
    use crate::member::config::MembersProviderConfig;
    use crate::member::tests::{get_expected_member, get_member_as_csv, MEMBERSHIP_NUMBER};
    use crate::tools::env_args::with_env_args;
    use crate::web::api::members_state::MembersState;
    use crate::web::api::members_controller::{download_members};
    use crate::tools::test::tests::temp_dir;

    // region download_members
    #[async_test]
    async fn should_download_members() {
        let mock_server = MockServer::start().await;

        let args = vec![
            "--login=test_login".to_string(),
            "--password=test_password".to_string(),
        ];
        let temp_dir = temp_dir();
        let config = MembersProviderConfig::new(
            mock_server.uri(),
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap(),
            temp_dir.into_os_string(),
        );
        let download_filename = "download.csv";
        let download_link = format!("{}/{download_filename}", mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(query_param_contains("P", "bo/extranet/adhesion/annuaire/index"))
            .and(body_string_contains("Action=adherent_filtrer"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/includer.php"))
            .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(format!("<p>Here is the download link: {download_link}</p>"), "text/html"))
            .mount(&mock_server)
            .await;
        let member_as_csv = get_member_as_csv();
        let member_as_csv = ISO_8859_1.encode(&member_as_csv, EncoderTrap::Strict).unwrap();
        let message_in_latin1: &[u8] = &member_as_csv;
        Mock::given(method("GET"))
            .and(path(format!("/{download_filename}").to_owned()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"))
            .mount(&mock_server)
            .await;

        let config_state = State::from(&config);
        let members_state_mutex = Mutex::new(MembersState::new(None, HashMap::new()));
        let members_state = State::from(&members_state_mutex);

        let result = with_env_args(args, || block_on(download_members(config_state, members_state))).unwrap();
        let members: Members = serde_json::from_str(&result).unwrap();
        assert_eq!(&get_expected_member(), members.get(MEMBERSHIP_NUMBER).unwrap().iter().find(|_| true).unwrap());
    }

    #[async_test]
    async fn should_not_download_members_when_error() {
        let mock_server = MockServer::start().await;

        let args = vec![];
        let temp_dir = temp_dir();
        let config = MembersProviderConfig::new(
            mock_server.uri(),
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap(),
            temp_dir.into_os_string(),
        );

        let config_state = State::from(&config);
        let members_state_mutex = Mutex::new(MembersState::new(None, HashMap::new()));
        let members_state = State::from(&members_state_mutex);

        let result = with_env_args(args, || block_on(download_members(config_state, members_state)));
        assert!(result.err().is_some());
    }
    // endregion
}
