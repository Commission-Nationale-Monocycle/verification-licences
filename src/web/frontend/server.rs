use crate::web::frontend::{filters, frontend_controller};
use crate::web::server::Server;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;

pub struct FrontendServer {}

impl FrontendServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for FrontendServer {
    fn configure(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        rocket_build
            .mount(
                "/",
                routes![
                    frontend_controller::index,
                    frontend_controller::fileo_login,
                    frontend_controller::update_memberships,
                    frontend_controller::update_memberships_unauthenticated,
                    frontend_controller::list_memberships,
                    frontend_controller::list_memberships_unauthenticated,
                    frontend_controller::look_membership_up,
                    frontend_controller::look_membership_up_unauthenticated,
                    frontend_controller::check_members_from_csv,
                    frontend_controller::check_members_from_csv_unauthenticated,
                    frontend_controller::check_members_from_uda,
                    frontend_controller::check_members_from_uda_unauthenticated,
                ],
            )
            .mount("/", FileServer::from("./public/static"))
            .register("/", catchers![frontend_controller::not_found])
            .attach(Template::custom(|engines| {
                engines
                    .tera
                    .register_filter("is_in_the_past", filters::is_in_the_past)
            }))
    }
}
