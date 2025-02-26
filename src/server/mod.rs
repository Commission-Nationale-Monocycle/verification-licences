use std::sync::Mutex;
use rocket::{Build, Rocket};
use crate::member::config::MembersProviderConfig;
use crate::server::members_state::MembersState;

pub mod members_state;
pub mod members_controller;

pub fn start_server(members_provider_config: MembersProviderConfig, members_state: MembersState) -> Rocket<Build> {
    rocket::build()
        .manage(members_provider_config)
        .manage(Mutex::new(members_state))
        .mount("/", routes![members_controller::list_members, members_controller::update_members])
}