mod alert;
mod card_creator;
mod error;
mod navbar;
mod stepper;
mod template;
mod user_interface;
mod utils;

use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::card_creator::EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME;
use crate::error::Error;
use crate::stepper::next_step;
use crate::user_interface::{get_email_body, get_email_subject, set_loading};
use crate::utils::{
    get_document, get_element_by_id_dyn, get_value_from_input, get_window,
    query_selector_single_element,
};
use dto::checked_member::CheckedMember;
use dto::email::Email;
use dto::member_to_check::MemberToCheck;
use reqwest::Client;
use serde_json::json;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlInputElement, HtmlTextAreaElement};

#[wasm_bindgen(start)]
fn run() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let document = &unwrap_without_alert(get_document());
    unwrap_or_alert(navbar::init_navbar(document));
}

// region Handle "members to check" file
#[wasm_bindgen]
pub async fn handle_members_to_check_file(input: HtmlInputElement) -> Result<(), JsValue> {
    unwrap_or_alert(set_loading(true));

    let document = unwrap_without_alert(get_document());

    let csv_file = input
        .files()
        .expect("no files")
        .get(0)
        .expect("file should be accessible");

    let promise = csv_file.text();
    let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let csv_content = text_jsvalue.as_string().unwrap_or_else(|| {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Le fichier CSV contient des caractères incorrects. Vérifiez l'encodage UTF-8 du fichier.",
            AlertLevel::Error,
        );
        panic!("csv file should contain only valid UTF-8 characters");
    });

    let (members_to_check, wrong_lines) =
        MemberToCheck::load_members_to_check_from_csv_string(&csv_content);

    unwrap_or_alert(user_interface::render_lines(
        &document,
        &csv_content,
        &members_to_check,
        &wrong_lines,
    ));

    unwrap_or_alert(set_loading(false));
    Ok(())
}

// endregion

// region Handle form submission
#[wasm_bindgen]
pub async fn handle_form_submission(document: &Document) {
    unwrap_or_alert(set_loading(true));
    let members_to_check_input =
        unwrap_or_alert(get_value_from_input(document, "members-to-check"));

    let client = build_client();

    let window = unwrap_without_alert(get_window());
    let origin = window.location().origin().unwrap();
    let url = format!("{origin}/api/members/check");
    let body = format!("members_to_check={members_to_check_input}");
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| {
            unwrap_or_alert(set_loading(false));
            create_alert(
                "Impossible d'envoyer la requête. Veuillez réessayer.",
                AlertLevel::Error,
            );
            panic!("can't send request: {error:?}")
        });

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        let text = response.text().await.expect("can't get text");
        let checked_members: Vec<CheckedMember> =
            serde_json::from_str(&text).expect("can't deserialize checked members");
        unwrap_or_alert(user_interface::handle_checked_members(&checked_members));
        next_step(document);
        unwrap_or_alert(set_loading(false));
    } else {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
            AlertLevel::Error,
        );
        log::error!("Server error: {}", response.status().as_str())
    }
}

#[wasm_bindgen]
pub fn go_to_notification_step(document: &Document) {
    let addresses_to_notify = unwrap_or_alert(get_email_addresses_to_notify(document));
    let text = addresses_to_notify.join("\n");
    let element = unwrap_or_alert(get_element_by_id_dyn::<HtmlTextAreaElement>(
        document,
        "email-recipients",
    ));
    element.set_value(&text);

    next_step(document);
}
// endregion

// region Handle email sending
#[wasm_bindgen]
pub async fn handle_email_sending() {
    unwrap_or_alert(set_loading(true));
    let document = &unwrap_without_alert(get_document());
    let email_addresses_to_notify = unwrap_or_alert(get_email_addresses_to_notify(document));
    let email_subject = unwrap_or_alert(get_email_subject(document));
    let email_body = unwrap_or_alert(get_email_body(document));

    let client = build_client();
    let origin = unwrap_without_alert(get_window())
        .location()
        .origin()
        .unwrap();
    let url = format!("{origin}/api/members/notify");
    let body = json!(Email::new(
        email_addresses_to_notify.clone(),
        email_subject.to_owned(),
        email_body.to_owned(),
    ))
    .to_string();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| {
            unwrap_or_alert(set_loading(false));
            create_alert(
                "Impossible d'envoyer la requête. Veuillez réessayer.",
                AlertLevel::Error,
            );
            panic!("can't send request: {error:?}")
        });

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        let addresses_count = email_addresses_to_notify.len();
        create_alert(
            &format!(
                "L'email a bien été envoyé à {} adresse{}.",
                &addresses_count,
                if addresses_count > 1 { "s" } else { "" }
            ),
            AlertLevel::Info,
        );
        log::info!("Email sent to {:?}!", email_addresses_to_notify);
        unwrap_or_alert(set_loading(false));
    } else {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Impossible d'envoyer l'email. Veuillez réessayer.",
            AlertLevel::Error,
        );
        log::error!("Server error: {}", response.status().as_str());
    }
}

fn get_email_addresses_to_notify(document: &Document) -> Result<Vec<String>> {
    let checked_members_container = user_interface::get_checked_members_container(document);
    let expired_members = checked_members_container?
        .get_elements_by_class_name(EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME);
    let mut email_addresses_to_notify = vec![];
    for index in 0..expired_members.length() {
        let expired_member = expired_members.get_with_index(index).unwrap();
        let checkboxes = expired_member.get_elements_by_tag_name("input");
        if checkboxes.length() != 1 {
            set_loading(false)?;
            create_alert(
                "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                AlertLevel::Error,
            );
            log::error!(
                "There should be a single checkbox [count: {}]",
                checkboxes.length()
            );
        } else {
            let checkbox = checkboxes
                .get_with_index(0)
                .unwrap()
                .dyn_into::<HtmlInputElement>()?;
            let is_checked = checkbox.checked();
            if is_checked {
                let address_container =
                    query_selector_single_element(&expired_member, ".email-address-container a")?;
                match address_container.text_content() {
                    None => set_loading(false)?,
                    Some(email_address) => email_addresses_to_notify.push(email_address),
                };
            }
        }
    }
    Ok(email_addresses_to_notify)
}
// endregion

fn build_client() -> Client {
    Client::builder().build().unwrap_or_else(|error| {
        create_alert(
            "Impossible d'envoyer la requête. Veuillez réessayer.",
            AlertLevel::Error,
        );
        panic!("could not build client: {error:?}")
    })
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
