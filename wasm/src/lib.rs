mod alert;
mod card_creator;
mod navbar;
mod user_interface;
mod utils;

use crate::alert::{AlertLevel, create_alert};
use crate::card_creator::EXPIRED_MEMBERSHIP_CONTAINER_CLASS_NAME;
use crate::user_interface::{get_email_body, get_email_subject};
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
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, Event, HtmlFormElement, HtmlInputElement};

#[wasm_bindgen(start)]
fn run() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let document = &get_document();
    navbar::init_navbar(document);
    add_submit_event_listener_to_form(document);
}

// region Handle "members to check" file
#[wasm_bindgen]
pub async fn handle_members_to_check_file(input: HtmlInputElement) -> Result<(), JsValue> {
    let document = get_document();

    let csv_file = input
        .files()
        .expect("no files")
        .get(0)
        .expect("file should be accessible");

    let promise = csv_file.text();
    let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let csv_content = text_jsvalue.as_string().unwrap_or_else(|| {
        create_alert(
            &document,
            "Le fichier CSV contient des caractères incorrects. Vérifiez l'encodage UTF-8 du fichier.",
            AlertLevel::Error,
        );
        panic!("csv file should contain only valid UTF-8 characters");
    });

    let (members_to_check, wrong_lines) =
        MemberToCheck::load_members_to_check_from_csv_string(&csv_content);

    user_interface::render_lines(&document, &csv_content, &members_to_check, &wrong_lines);

    Ok(())
}

// endregion

// region Handle form submission
fn add_submit_event_listener_to_form(document: &Document) {
    let form = get_element_by_id_dyn::<HtmlFormElement>(&document, "check_members_form");
    let closure = Closure::wrap(Box::new(|e: Event| {
        spawn_local(async move {
            handle_form_submission(e).await;
        });
    }) as Box<dyn Fn(_)>);
    form.add_event_listener_with_event_listener("submit", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

async fn handle_form_submission(e: Event) {
    e.prevent_default();
    let document = get_document();
    let members_to_check_input = get_value_from_input(&document, "members_to_check");

    let client = build_client();

    let origin = get_window().location().origin().unwrap();
    let url = format!("{origin}/api/members/check");
    let body = format!("members_to_check={members_to_check_input}");
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| {
            create_alert(
                &document,
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
        user_interface::handle_checked_members(&checked_members);
    } else {
        create_alert(
            &document,
            "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
            AlertLevel::Error,
        );
        log::error!("Server error: {}", response.status().as_str())
    }
}

fn build_client() -> Client {
    Client::builder().build().unwrap_or_else(|error| {
        create_alert(
            &get_document(),
            "Impossible d'envoyer la requête. Veuillez réessayer.",
            AlertLevel::Error,
        );
        panic!("could not build client: {error:?}")
    })
}
// endregion

// region Handle email sending
#[wasm_bindgen]
pub async fn handle_email_sending() {
    let document = &get_document();
    let email_addresses_to_notify = get_email_addresses_to_notify(document);
    let email_subject = get_email_subject(document);
    let email_body = get_email_body(document);

    let client = build_client();
    let origin = get_window().location().origin().unwrap();
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
            create_alert(
                document,
                "Impossible d'envoyer la requête. Veuillez réessayer.",
                AlertLevel::Error,
            );
            panic!("can't send request: {error:?}")
        });

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        let addresses_count = email_addresses_to_notify.len();
        create_alert(
            document,
            &format!(
                "L'email a bien été envoyé à {} adresse{}.",
                &addresses_count,
                if addresses_count > 1 { "s" } else { "" }
            ),
            AlertLevel::Info,
        );
        log::info!("Email sent to {:?}!", email_addresses_to_notify);
    } else {
        create_alert(
            document,
            "Impossible d'envoyer l'email. Veuillez réessayer.",
            AlertLevel::Error,
        );
        log::error!("Server error: {}", response.status().as_str());
    }
}

fn get_email_addresses_to_notify(document: &Document) -> Vec<String> {
    let checked_members_container = user_interface::get_checked_members_container(document);
    let expired_members = checked_members_container
        .get_elements_by_class_name(EXPIRED_MEMBERSHIP_CONTAINER_CLASS_NAME);
    let mut email_addresses_to_notify = vec![];
    for index in 0..expired_members.length() {
        let expired_member = expired_members.get_with_index(index).unwrap();
        let checkboxes = expired_member.get_elements_by_tag_name("input");
        if checkboxes.length() != 1 {
            create_alert(
                document,
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
                .dyn_into::<HtmlInputElement>()
                .unwrap();
            let is_checked = checkbox.checked();
            if is_checked {
                let address_container = query_selector_single_element(
                    document,
                    &expired_member,
                    ".email-address-container a",
                );
                let email_address = address_container.text_content().unwrap_or_else(|| {
                    create_alert(
                        document,
                        "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                        AlertLevel::Error,
                    );
                    panic!("There should be a single email address in each box.")
                });
                email_addresses_to_notify.push(email_address);
            }
        }
    }
    email_addresses_to_notify
}
// endregion
