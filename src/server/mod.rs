use std::sync::Mutex;
use rocket::{Build, Rocket};
use crate::server::members_state::MembersState;

pub mod members_state;
pub mod members_controller;

pub fn start_server(members_state: MembersState) -> Rocket<Build> {
    rocket::build()
        .manage(Mutex::new(members_state))
        .mount("/", routes![members_controller::members, members_controller::update_members])
}