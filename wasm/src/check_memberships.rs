use crate::Result;
use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::card_creator::EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME;
use crate::error::{DEFAULT_ERROR_MESSAGE, DEFAULT_SERVER_ERROR_MESSAGE, Error};
use crate::stepper::next_step;
use crate::user_interface::{
    get_email_body, get_email_subject, get_members_to_check_hidden_input, set_loading, with_loading,
};
use crate::utils::{
    get_document, get_element_by_id_dyn, get_value_from_element, query_selector_single_element,
};
use crate::web::fetch;
use crate::{json, user_interface};
use dto::checked_member::CheckedMember;
use dto::csv_member::CsvMember;
use dto::email::Email;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement};

// region Handle "members to check" file
#[wasm_bindgen]
pub async fn handle_members_to_check_file(input: HtmlInputElement) {
    with_loading(async || {
        let document = get_document()?;

        let csv_file = input
            .files()
            .expect("no files")
            .get(0)
            .expect("file should be accessible");

        let promise = csv_file.text();
        let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
        let csv_content = text_jsvalue.as_string().ok_or_else(|| Error::new("Le fichier CSV contient des caractères incorrects. Vérifiez l'encodage UTF-8 du fichier.", "csv file should contain only valid UTF-8 characters"))?;

        let (members_to_check, wrong_lines) =
            CsvMember::load_members_to_check_from_csv_string(&csv_content);

        user_interface::render_lines(
            &document,
            &csv_content,
            &members_to_check,
            &wrong_lines,
        )
    }).await;
}

// endregion

// region Handle form submission
#[wasm_bindgen]
pub async fn handle_form_submission(document: &Document) {
    with_loading(async || {
        let members_to_check_input = get_members_to_check_hidden_input(document)?;
        let members_to_check = get_value_from_element(&members_to_check_input);
        if members_to_check.trim().is_empty() {
            return Err(Error::new(
                "Impossible de valider un fichier vide. Veuillez réessayer.",
                "Can't check an empty file.",
            ));
        }

        let url = "/api/members/csv/check";
        let body = format!("members_to_check={members_to_check}");
        let response = fetch(
            url,
            "post",
            Some("application/x-www-form-urlencoded"),
            Some(&body),
        )
        .await
        .map_err(|error| Error::from_parent(DEFAULT_SERVER_ERROR_MESSAGE, error))?;

        let status = response.status();
        if (200..400).contains(&status) {
            let text = response.body().clone().unwrap_or(String::new());
            let checked_members: Vec<CheckedMember<CsvMember>> = json::from_str(&text);
            user_interface::handle_checked_members(&checked_members)?;
            next_step(document);

            Ok(())
        } else {
            Err(Error::from_server_status_error(status))
        }
    })
    .await;
}

#[wasm_bindgen]
pub fn toggle_next_step_button() {
    let document = unwrap_without_alert(get_document());
    let email_addresses_to_notify = unwrap_or_alert(get_email_addresses_to_notify(&document));

    let button = unwrap_or_alert(get_element_by_id_dyn::<HtmlButtonElement>(
        &document,
        "go-to-send-email-step",
    ));
    button.set_disabled(email_addresses_to_notify.is_empty());
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
    with_loading(async || {
        let document = &get_document()?;
        let email_addresses_to_notify = get_email_addresses_to_notify(document)?;
        let email_subject = get_email_subject(document)?;
        let email_body = get_email_body(document)?;

        let url = "/api/members/notify";
        let email = Email::new(
            email_addresses_to_notify.clone(),
            email_subject.to_owned(),
            email_body.to_owned(),
        );
        let body = json::to_string(&email);

        let response = fetch(url, "post", Some("application/json"), Some(&body))
            .await
            .map_err(|error| Error::from_parent(DEFAULT_SERVER_ERROR_MESSAGE, error))?;
        let status = response.status();
        if (200..400).contains(&status) {
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

            Ok(())
        } else {
            Err(Error::from_server_status_error(status))
        }
    })
    .await;
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
            Err(Error::new(
                DEFAULT_ERROR_MESSAGE,
                &format!(
                    "There should be a single checkbox [count: {}]",
                    checkboxes.length()
                ),
            ))?
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
