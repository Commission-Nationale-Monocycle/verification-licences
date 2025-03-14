use crate::member::members::Members;
use crate::tools::email::send_email;
use crate::tools::log_message_and_return;
use crate::web::api::members_state::MembersState;
use crate::web::credentials::FileoCredentials;
use dto::email::Email;
use dto::member_to_check::MemberToCheck;
use rocket::State;
use rocket::form::Form;
use rocket::serde::json::Json;
use serde_json::json;
use std::sync::Mutex;

/// Check members as a CSV whose columns are:
/// ```
/// membership_number;lastname;firstname
/// ```
/// Return the result as JSON-encoded string,
/// within which each member having a valid membership has its last occurrence associated,
/// while each member having no valid membership has no element associated.
#[post("/members/check", data = "<members_to_check>")]
pub async fn check_memberships(
    members_state: &State<Mutex<MembersState>>,
    members_to_check: Form<String>,
    _credentials: FileoCredentials,
) -> Result<String, String> {
    let members_to_check =
        match MemberToCheck::load_members_to_check_from_csv_string(&members_to_check) {
            (members_to_check, wrong_lines) if wrong_lines.is_empty() => members_to_check,
            (members_to_check, wrong_lines) => {
                wrong_lines.iter().for_each(|wrong_line| {
                    error!("Line couldn't be read: {wrong_line}");
                });
                members_to_check
            }
        };
    let members_state = members_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        "Error while checking members.",
    ))?;

    let members: &Members = members_state.members();
    let members_to_check = members_to_check.into_iter().collect::<Vec<_>>();
    let vec = members.check_members(&members_to_check);

    Ok(json!(vec).to_string())
}

/// Email all recipients specified as argument.
#[post("/members/notify", format = "application/json", data = "<email>")]
pub async fn notify_members(
    email: Json<Email>,
    _credentials: FileoCredentials,
) -> Result<(), String> {
    let recipients = email
        .recipients()
        .iter()
        .map(|email| email.as_ref())
        .collect::<Vec<&str>>();
    send_email(recipients.as_ref(), email.subject(), email.body())
        .await
        .map_err(log_message_and_return(
            "Couldn't send email",
            "Email has not been sent.",
        ))?;

    Ok(())
}
