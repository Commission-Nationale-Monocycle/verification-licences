#[macro_use]
extern crate rocket;
#[cfg(feature = "demo")]
mod demo_mock_server;
mod error;
mod fileo;
mod member;
mod tools;
mod uda;
mod web;

#[cfg(feature = "demo")]
use crate::demo_mock_server::init_demo;
use crate::web::start_servers;

#[launch]
async fn rocket() -> _ {
    env_logger::init();
    #[cfg(feature = "demo")]
    init_demo().await;
    start_servers()
}
