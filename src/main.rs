mod member;
mod server;
mod tools;

#[macro_use]
extern crate rocket;

use regex::Regex;
use crate::member::config::MembersProviderConfig;
use crate::member::get_members_file_folder;
use crate::member::import_from_file::find_file;
use crate::server::members_state::MembersState;
use crate::server::start_server;

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let members_provider_config = build_members_provider_config();
    let members_state = load_members_file_details();
    start_server(members_provider_config, members_state)
}

fn build_members_provider_config() -> MembersProviderConfig {
    MembersProviderConfig::new(
        "https://www.leolagrange-fileo.org".to_owned(),
        Regex::new("https://www.leolagrange-fileo.org/clients/fll/telechargements/temp/.*?\\.csv").unwrap(),
        get_members_file_folder().to_os_string()
    )
}

fn load_members_file_details() -> MembersState {
    match find_file() {
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
