use rocket::{Build, Rocket};
use crate::web::server::Server;

pub struct FrontendServer {

}

impl FrontendServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for FrontendServer {
    fn initialize_managed_states(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        rocket_build
    }

    fn mount_routes(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        rocket_build.mount("/", routes![])
    }
}
