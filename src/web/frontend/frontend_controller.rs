use std::sync::Mutex;
use rocket::{Request, State};
use rocket::response::Redirect;

use rocket_dyn_templates::{context, Template};
use crate::member::MembershipDto;
use crate::member::members::Members;
use crate::member::memberships::Memberships;
use crate::web::api::members_state::MembersState;

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to(uri!("/", hello(name = "Your Name")))
}

#[get("/hello/<name>")]
pub async fn hello(name: &str) -> Template {
    Template::render("index", context! {
        title: "Hello",
        name: Some(name),
        items: vec!["One", "Two", "Three"],
    })
}

#[get("/memberships")]
pub async fn list_memberships(members_state: &State<Mutex<MembersState>>) -> Template {
    let members = members_state.lock().unwrap();
    let members: &Members = members.members();
    let memberships: Vec<&MembershipDto> = members.values()
        .filter_map(Memberships::find_last_membership)
        .collect();

    Template::render("memberships", context! {
        title: "Liste des licences",
        memberships: memberships
    })
}

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}
