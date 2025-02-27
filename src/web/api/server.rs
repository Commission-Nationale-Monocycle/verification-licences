use std::ffi::OsStr;
use std::sync::Mutex;
use regex::Regex;
use rocket::{Build, Rocket};
use crate::member;
use crate::web::api::members_controller;
use crate::web::api::members_state::MembersState;
use crate::member::config::MembersProviderConfig;
use crate::member::get_members_file_folder;
use crate::member::import_from_file::find_file;
use crate::web::server::Server;

pub struct ApiServer {}

impl ApiServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for ApiServer {
    fn configure(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        let members_provider_config = build_members_provider_config();
        let members_state = load_members_file_details(members_provider_config.folder());

        rocket_build
            .manage(members_provider_config)
            .manage(Mutex::new(members_state))
            .mount("/api/", routes![
                members_controller::list_members,
                members_controller::update_members
            ])
    }
}

fn build_members_provider_config() -> MembersProviderConfig {
    MembersProviderConfig::new(
        "https://www.leolagrange-fileo.org".to_owned(),
        Regex::new("https://www.leolagrange-fileo.org/clients/fll/telechargements/temp/.*?\\.csv").unwrap(),
        get_members_file_folder().to_os_string(),
    )
}

fn load_members_file_details(members_file_folder: &OsStr) -> MembersState {
    match find_file(members_file_folder) {
        Ok(file_details) => {
            MembersState::new(Some(file_details))
        }
        Err(member::error::Error::NoFileFound) => { MembersState::default() }
        Err(e) => {
            error!("Can't read members file, aborting...\n{e:#?}");
            panic!();
        }
    }
}
