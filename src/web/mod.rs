use rocket::{Build, Rocket};
use crate::web::server::build_server;

mod api;
mod frontend;
mod server;

pub fn start_servers() -> Rocket<Build> {
    build_server()
}
