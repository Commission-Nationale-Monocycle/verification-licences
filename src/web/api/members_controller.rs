use std::sync::Mutex;
use rocket::State;
use serde_json::json;
use crate::member::config::MembershipsProviderConfig;
use crate::member::download::download_memberships_list;
use crate::member::import_from_file::{clean_old_files, import_from_file};
use crate::web::api::members_state::MembersState;
use crate::tools::{log_message, log_message_and_return};

/// Download memberships csv file from remote provided in config,
/// write said file into filesystem
/// and load it into memory.
/// Finally, clean all old memberships files.
#[get("/memberships")]
pub async fn download_memberships(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    members_state: &State<Mutex<MembersState>>,
) -> Result<String, String> {
    let file_details = download_memberships_list(memberships_provider_config)
        .await
        .map_err(log_message_and_return("Can't download memberships list", "Can't download memberships list"))?;

    let members = import_from_file(file_details.filepath())
        .map_err(log_message_and_return("Error while reading memberships file.", "Error while reading memberships file."))?;
    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while updating memberships."))?;

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
    use std::fs;
    use std::sync::Mutex;
    use encoding::all::ISO_8859_1;
    use encoding::{EncoderTrap, Encoding};
    use regex::Regex;
    use rocket::futures::executor::block_on;
    use rocket::State;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
    use crate::member::members::Members;
    use crate::member::config::MembershipsProviderConfig;
    use crate::member::tests::{get_expected_member, get_member_as_csv, MEMBERSHIP_NUMBER};
    use crate::tools::env_args::with_env_args;
    use crate::web::api::members_state::MembersState;
    use crate::web::api::members_controller::download_memberships;
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
        let old_file_path = temp_dir.join("memberships-1980-01-01.csv");
        let config = MembershipsProviderConfig::new(
            mock_server.uri(),
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap(),
            temp_dir.into_os_string(),
        );
        let download_filename = "download.csv";
        let download_link = format!("{}/{download_filename}", mock_server.uri());

        fs::write(&old_file_path, "").unwrap();
        assert!(fs::exists(&old_file_path).ok().unwrap());

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
        let members_state_mutex = Mutex::new(MembersState::new(None, Members::default()));
        let members_state = State::from(&members_state_mutex);

        let result = with_env_args(args, || block_on(download_memberships(config_state, members_state))).unwrap();
        let members: Members = serde_json::from_str(&result).unwrap();
        assert_eq!(&get_expected_member(), members.get(MEMBERSHIP_NUMBER).unwrap().iter().find(|_| true).unwrap());
        assert!(!fs::exists(&old_file_path).ok().unwrap(), "Old file should have been cleaned.");
    }

    #[async_test]
    async fn should_not_download_members_when_error() {
        let mock_server = MockServer::start().await;

        let args = vec![];
        let temp_dir = temp_dir();
        let config = MembershipsProviderConfig::new(
            mock_server.uri(),
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap(),
            temp_dir.into_os_string(),
        );

        let config_state = State::from(&config);
        let members_state_mutex = Mutex::new(MembersState::new(None, Members::default()));
        let members_state = State::from(&members_state_mutex);

        let result = with_env_args(args, || block_on(download_memberships(config_state, members_state)));
        assert!(result.err().is_some());
    }
    // endregion
}
