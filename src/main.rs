#[macro_use]
extern crate rocket;
pub(crate) mod database;
#[cfg(feature = "demo")]
mod demo_mock_server;
mod error;
mod fileo;
mod membership;
mod tools;
mod uda;
mod web;

use crate::database::init_db;
#[cfg(feature = "demo")]
use crate::demo_mock_server::init_demo;
use crate::web::start_servers;

#[launch]
async fn rocket() -> _ {
    env_logger::init();
    init_db().expect("Failed to initialize database");
    #[cfg(feature = "demo")]
    init_demo().await;
    start_servers()
}
