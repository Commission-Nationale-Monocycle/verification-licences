#[macro_use]
extern crate rocket;
pub(crate) mod database;
#[cfg(feature = "demo")]
mod demo_mock_server;
mod error;
mod fileo;
mod membership;
mod notification;
mod tools;
mod uda;
mod web;

use crate::database::init_connection_pool;
#[cfg(feature = "demo")]
use crate::demo_mock_server::init_demo;
use crate::web::start_servers;

#[launch]
async fn rocket() -> _ {
    env_logger::init();
    let pool = init_connection_pool().expect("Failed to initialize database connection pool");
    #[cfg(feature = "demo")]
    init_demo().await;
    start_servers(pool)
}
