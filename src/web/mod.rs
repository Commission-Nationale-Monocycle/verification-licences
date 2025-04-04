use crate::web::server::build_server;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use rocket::{Build, Rocket};

mod api;
pub(crate) mod authentication;
pub mod credentials_storage;
pub mod error;
mod frontend;
mod server;

pub fn start_servers(pool: Pool<ConnectionManager<SqliteConnection>>) -> Rocket<Build> {
    build_server(pool)
}
