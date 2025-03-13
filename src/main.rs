#[macro_use]
extern crate rocket;
mod member;
mod tools;
mod uda;
mod web;

use crate::web::start_servers;

#[launch]
fn rocket() -> _ {
    env_logger::init();
    start_servers()
}
