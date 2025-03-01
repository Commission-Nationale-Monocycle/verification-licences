mod member;
mod tools;
mod web;

#[macro_use]
extern crate rocket;

use crate::web::start_servers;

#[launch]
fn rocket() -> _ {
    env_logger::init();
    start_servers()
}
