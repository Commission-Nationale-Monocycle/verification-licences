use crate::web::server::build_server;
use rocket::{Build, Rocket};

mod api;
mod authentication;
pub mod credentials;
pub mod error;
mod frontend;
mod server;

pub fn start_servers() -> Rocket<Build> {
    build_server()
}
