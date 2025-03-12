use rocket::{Request, State};
use std::sync::Mutex;

use crate::member::members::Members;
use crate::member::memberships::Memberships;
use crate::web::api::members_state::MembersState;
use dto::membership::Membership;
use rocket_dyn_templates::{Template, context};

#[get("/memberships")]
pub async fn list_memberships(members_state: &State<Mutex<MembersState>>) -> Template {
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

#[get("/check-memberships")]
pub async fn check_memberships() -> Template {
    Template::render(
        "check-memberships",
        context! {
            title: "Vérifier les licences"
        },
    )
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
