use crate::fileo::credentials::FileoCredentials;
use crate::tools::email::send_email;
use crate::tools::log_message_and_return;
use crate::uda::credentials::UdaCredentials;
use crate::web::api::memberships_state::MembershipsState;
use dto::checked_member::CheckedMember;
use dto::csv_member::CsvMember;
use dto::email::Email;
use dto::member_identifier::MemberIdentifier;
use dto::uda_member::UdaMember;
use rocket::State;
use rocket::form::Form;
use rocket::serde::json::{Json, json};
use std::sync::Mutex;

/// Check members as a CSV whose columns are:
/// ```
/// membership_number;lastname;firstname
/// ```
/// Return the result as JSON-encoded string,
/// within which each member having a valid membership has its last occurrence associated,
/// while each member having no valid membership has no element associated.
#[post("/members/csv/check", data = "<members_to_check>")]
pub async fn check_csv_members(
    memberships_state: &State<Mutex<MembershipsState>>,
    members_to_check: Form<String>,
    _credentials: FileoCredentials,
) -> Result<String, String> {
    let members_to_check = match CsvMember::load_members_to_check_from_csv_string(&members_to_check)
    {
        (members_to_check, wrong_lines) if wrong_lines.is_empty() => members_to_check,
        (members_to_check, wrong_lines) => {
            wrong_lines.iter().for_each(|wrong_line| {
                error!("Line couldn't be read: {wrong_line}");
            });
            members_to_check
        }
    };

    let result = check(memberships_state, members_to_check.into_iter().collect())?;

    Ok(json!(result).to_string())
}

#[post("/members/uda/check", data = "<members_to_check>")]
pub async fn check_uda_members(
    memberships_state: &State<Mutex<MembershipsState>>,
    members_to_check: Json<Vec<UdaMember>>,
    _fileo_credentials: FileoCredentials,
    _uda_credentials: UdaCredentials,
) -> Result<String, String> {
    let result = check(memberships_state, members_to_check.into_inner())?;

    Ok(json!(result).to_string())
}

fn check<T: MemberIdentifier>(
    memberships_state: &Mutex<MembershipsState>,
    members_to_check: Vec<T>,
) -> Result<Vec<CheckedMember<T>>, String> {
    let memberships_state = memberships_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        "Error while checking members.",
    ))?;

    let memberships = memberships_state.memberships();
    let checked_members = memberships.check_members(members_to_check);

    Ok(checked_members)
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

#[cfg(test)]
mod tests {
    mod check_members {
        use crate::fileo::credentials::FileoCredentials;
        use crate::membership::grouped_memberships::GroupedMemberships;
        use crate::membership::memberships::Memberships;
        use crate::uda::credentials::UdaCredentials;
        use crate::web::api::memberships_controller::check_uda_members;
        use crate::web::api::memberships_state::MembershipsState;
        use crate::web::credentials_storage::CredentialsStorage;
        use dto::checked_member::CheckedMember;
        use dto::membership::tests::get_expected_membership;
        use dto::uda_member::UdaMember;
        use rocket::http::hyper::header::CONTENT_TYPE;
        use rocket::http::{ContentType, Header, Status};
        use rocket::local::asynchronous::Client;
        use rocket::serde::json::json;
        use std::collections::HashMap;
        use std::sync::Mutex;

        #[async_test]
        async fn success() {
            let member_1 = UdaMember::new(
                1,
                Some("123456".to_owned()),
                "Jon".to_owned(),
                "Doe".to_owned(),
                "jon.doe@email.com".to_owned(),
                Some("Le club de test".to_owned()),
                true,
            );
            let member_2 = UdaMember::new(
                2,
                Some("654321".to_owned()),
                "Jonette".to_owned(),
                "Snow".to_owned(),
                "jonette.snow@email.com".to_owned(),
                None,
                false,
            );
            let members = vec![member_1.clone(), member_2.clone()];

            let fileo_credentials =
                FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
            let uda_credentials = UdaCredentials::new(
                "https://test.reg.unicycling-software.com".to_owned(),
                "login@test.com".to_owned(),
                "password".to_owned(),
            );

            let fileo_uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
            let uda_uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e";
            let mut fileo_storage = CredentialsStorage::<FileoCredentials>::default();
            let mut uda_storage = CredentialsStorage::<UdaCredentials>::default();
            fileo_storage.store(fileo_uuid.to_string(), fileo_credentials);
            uda_storage.store(uda_uuid.to_string(), uda_credentials);

            let fileo_credentials_storage_mutex = Mutex::new(fileo_storage);
            let uda_credentials_storage_mutex = Mutex::new(uda_storage);

            let mut memberships = HashMap::new();
            memberships.insert(
                "123456".to_owned(),
                Memberships::from([get_expected_membership()]),
            );
            let memberships_state =
                MembershipsState::new(None, GroupedMemberships::from(memberships));
            let memberships_state = Mutex::new(memberships_state);

            let rocket = rocket::build()
                .manage(fileo_credentials_storage_mutex)
                .manage(uda_credentials_storage_mutex)
                .manage(memberships_state)
                .mount("/", routes![check_uda_members]);

            let client = Client::tracked(rocket).await.unwrap();
            let request = client
                .post("/members/uda/check")
                .cookie((
                    crate::fileo::authentication::AUTHENTICATION_COOKIE,
                    fileo_uuid,
                ))
                .cookie((crate::uda::authentication::AUTHENTICATION_COOKIE, uda_uuid))
                .body(json!(members).to_string().as_bytes())
                .header(Header::new(
                    CONTENT_TYPE.to_string(),
                    ContentType::JSON.to_string(),
                ));

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());

            let checked_members: Vec<CheckedMember<UdaMember>> =
                response.into_json().await.unwrap();
            assert_eq!(
                vec![
                    CheckedMember::new(member_1, Some(get_expected_membership())),
                    CheckedMember::new(member_2, None),
                ],
                checked_members
            )
        }
    }
}
