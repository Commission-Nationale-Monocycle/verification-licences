use crate::web::server::build_server;
use rocket::{Build, Rocket};

mod api;
pub(crate) mod authentication;
pub mod credentials_storage;
pub mod error;
mod frontend;
mod server;

pub fn start_servers() -> Rocket<Build> {
    build_server()
}
