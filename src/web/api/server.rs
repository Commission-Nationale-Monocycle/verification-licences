use crate::fileo::credentials::FileoCredentials;
use crate::membership::config::MembershipsProviderConfig;
use crate::uda::credentials::UdaCredentials;
use crate::web::api::{fileo_controller, memberships_controller, uda_controller};
use crate::web::credentials_storage::CredentialsStorage;
use crate::web::server::Server;
use dto::uda_instance::InstancesList;
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

        rocket_build
            .manage(members_provider_config)
            .manage(build_uda_configuration())
            .manage(Mutex::new(CredentialsStorage::<FileoCredentials>::default()))
            .manage(Mutex::new(CredentialsStorage::<UdaCredentials>::default()))
            .manage(Mutex::new(InstancesList::default()))
            .mount(
                "/api/",
                routes![
                    memberships_controller::check_csv_members,
                    memberships_controller::check_uda_members,
                    memberships_controller::notify_members,
                    memberships_controller::look_member_up,
                    fileo_controller::login,
                    fileo_controller::download_memberships,
                    uda_controller::login,
                    uda_controller::retrieve_members_to_check,
                    uda_controller::confirm_members,
                    uda_controller::list_instances,
                ],
            )
    }
}

fn build_members_provider_config() -> MembershipsProviderConfig {
    MembershipsProviderConfig::new(get_fileo_host(), get_download_link_regex())
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

#[cfg(not(feature = "demo"))]
fn build_uda_configuration() -> uda_connector::configuration::Configuration {
    uda_connector::configuration::Configuration::new(
        "https://reg.unicycling-software.com/tenants?locale=en".to_owned(),
    )
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

#[cfg(feature = "demo")]
fn build_uda_configuration() -> uda_connector::configuration::Configuration {
    let server_url = crate::demo_mock_server::UDA_MOCK_SERVER_URI
        .get()
        .unwrap()
        .clone();
    let url = format!("{server_url}/tenants?locale=en");

    uda_connector::configuration::Configuration::new(url)
}
