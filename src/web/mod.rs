use crate::notification::scheduler::events_notification_fairing;
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
    let rocket = build_server(pool.clone());
    if cfg!(feature = "demo") {
        // In demo mode, no real notification should be sent.
        rocket
    } else {
        rocket.attach(events_notification_fairing(pool))
    }
}
