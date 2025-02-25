mod member;
mod server;
mod tools;

#[macro_use]
extern crate rocket;

use crate::member::import_from_file::find_file;
use crate::server::members_state::MembersState;
use crate::server::start_server;

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let members_state = load_members_file_details();
    start_server(members_state)
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
