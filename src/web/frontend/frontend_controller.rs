use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;
use rocket::{Request, State};
use rocket::response::Redirect;

use rocket_dyn_templates::{Template, context};
use crate::member::MemberDto;
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

#[get("/members")]
pub async fn list_members(members_state: &State<Mutex<MembersState>>) -> Template {
    let members = members_state.lock().unwrap();
    let members: &HashMap<String, BTreeSet<MemberDto>> = members.members();
    let members: Vec<&MemberDto> = members.values()
        .map(|member_licences| member_licences.iter().max().unwrap())
        .collect();

    Template::render("members", context! {
        title: "Liste des membres",
        members: members
    })
}

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}
