use crate::member::config::MembershipsProviderConfig;
use crate::member::get_members_file_folder;
use crate::web::api::members_state::MembersState;
use crate::web::api::{fileo_controller, memberships_controller, uda_controller};
use crate::web::credentials::{CredentialsStorage, FileoCredentials, UdaCredentials};
use crate::web::server::Server;
use regex::Regex;
use rocket::{Build, Rocket};
use std::sync::Mutex;

pub struct ApiServer {}

impl ApiServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for ApiServer {
    fn configure(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        let members_provider_config = build_members_provider_config();
        let members_state = match MembersState::load_members(members_provider_config.folder()) {
            Ok(state) => state,
            Err(error) => {
                error!("{error:#?}");
                panic!("Initialization failed, aborting.");
            }
        };

        rocket_build
            .manage(members_provider_config)
            .manage(Mutex::new(members_state))
            .manage(Mutex::new(CredentialsStorage::<FileoCredentials>::default()))
            .manage(Mutex::new(CredentialsStorage::<UdaCredentials>::default()))
            .mount(
                "/api/",
                routes![
                    memberships_controller::check_memberships,
                    memberships_controller::notify_members,
                    fileo_controller::login,
                    fileo_controller::download_memberships,
                    uda_controller::login,
                    uda_controller::retrieve_members_to_check,
                    uda_controller::confirm_members,
                ],
            )
    }
}

fn build_members_provider_config() -> MembershipsProviderConfig {
    MembershipsProviderConfig::new(
        get_fileo_host(),
        get_download_link_regex(),
        get_members_file_folder().to_os_string(),
    )
}

#[cfg(not(feature = "demo"))]
fn get_fileo_host() -> String {
    "https://www.leolagrange-fileo.org".to_owned()
}

#[cfg(not(feature = "demo"))]
fn get_download_link_regex() -> Regex {
    Regex::new("https://www.leolagrange-fileo.org/clients/fll/telechargements/temp/.*?\\.csv")
        .unwrap()
}

#[cfg(feature = "demo")]
fn get_fileo_host() -> String {
    crate::demo_mock_server::FILEO_MOCK_SERVER_URI
        .get()
        .unwrap()
        .clone()
}

#[cfg(feature = "demo")]
fn get_download_link_regex() -> Regex {
    Regex::new("http://.*?\\.csv").unwrap()
}
