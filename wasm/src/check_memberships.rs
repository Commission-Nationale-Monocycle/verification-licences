use crate::Result;
use crate::component::alert::{AlertLevel, create_alert, unwrap_or_alert};
use crate::component::stepper::next_step;
use crate::error::{DEFAULT_SERVER_ERROR_MESSAGE, Error};
use crate::json;
use crate::user_interface::{get_email_body, get_email_subject, set_loading, with_loading};
use crate::utils::{get_document, get_element_by_id_dyn, query_selector_single_element};
use crate::web::fetch;
use dto::email::Email;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement};

// region Handle steps
#[wasm_bindgen]
pub fn toggle_go_to_email_step_button(document: &Document) {
    let email_addresses_to_notify = unwrap_or_alert(get_email_addresses_to_notify(document));

    let button = unwrap_or_alert(get_element_by_id_dyn::<HtmlButtonElement>(
        document,
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

            Ok(())
        } else {
            Err(Error::from_server_status_error(status))
        }
    })
    .await;
}

fn get_email_addresses_to_notify(document: &Document) -> Result<Vec<String>> {
    // let checked_members_container = user_interface::get_checked_members_container(document);
    let memberships = document.get_elements_by_class_name("membership");
    let mut email_addresses_to_notify = vec![];
    for index in 0..memberships.length() {
        let membership = memberships.get_with_index(index).unwrap();
        if membership.class_name().contains("membership-unknown") {
            continue;
        }

        let checkbox = query_selector_single_element(&membership, "input[type=\"checkbox\"]")?
            .dyn_into::<HtmlInputElement>()?;
        let is_checked = checkbox.checked();
        if is_checked {
            let address_container =
                query_selector_single_element(&membership, ".email-address-container a")?;
            match address_container.text_content() {
                None => set_loading(false)?,
                Some(email_address) => email_addresses_to_notify.push(email_address),
            };
        }
    }
    Ok(email_addresses_to_notify)
}
// endregion
