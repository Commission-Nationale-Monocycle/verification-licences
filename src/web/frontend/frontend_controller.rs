use crate::member::members::Members;
use crate::member::memberships::Memberships;
use crate::web::api::members_state::MembersState;
use crate::web::credentials::Credentials;
use dto::membership::Membership;
use rocket::response::Redirect;
use rocket::{Request, State};
use rocket_dyn_templates::{Template, context};
use std::sync::Mutex;

#[get("/fileo/login")]
pub async fn fileo_login() -> Template {
    Template::render(
        "fileo-login",
        context! {
            title: "Connexion à Fileo"
        },
    )
}

#[get("/memberships")]
pub async fn list_memberships(
    members_state: &State<Mutex<MembersState>>,
    _credentials: Credentials,
) -> Template {
    let members = members_state.lock().unwrap();
    let members: &Members = members.members();
    let memberships: Vec<&Membership> = members
        .values()
        .filter_map(Memberships::find_last_membership)
        .collect();

    Template::render(
        "memberships",
        context! {
            title: "Liste des licences",
            memberships: memberships
        },
    )
}

#[get("/memberships", rank = 2)]
pub async fn list_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=memberships"))
}

#[get("/check-memberships")]
pub async fn check_memberships(_credentials: Credentials) -> Template {
    Template::render(
        "check-memberships",
        context! {
            title: "Vérifier les licences"
        },
    )
}

#[get("/check-memberships", rank = 2)]
pub async fn check_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=check-memberships"))
}

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> Template {
    Template::render(
        "error/404",
        context! {
            uri: req.uri()
        },
    )
}
