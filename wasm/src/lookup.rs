use crate::Result;
use crate::card_creator::create_known_membership_card;
use crate::check_memberships::toggle_go_to_email_step_button;
use crate::component::alert::unwrap_or_alert;
use crate::component::login_form::add_enter_listener_on_form;
use crate::component::stepper::{add_step, next_step};
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::json;
use crate::user_interface::with_loading;
use crate::utils::{
    append_child, clear_element, get_element_by_id, get_element_by_id_dyn, get_value_from_element,
};
use crate::web::fetch;
use dto::member_to_look_up::MemberToLookUp;
use dto::membership::Membership;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Document;

pub fn init_lookup_page(document: &Document) {
    if let Some(stepper) = document
        .get_elements_by_class_name("stepper")
        .get_with_index(0)
    {
        unwrap_or_alert(add_step(document, &stepper, "Critères"));
        unwrap_or_alert(add_step(document, &stepper, "Résultats"));
        unwrap_or_alert(add_step(document, &stepper, "Notification"));
    }

    add_enter_listener_on_form(document, "lookup-form");
}

#[wasm_bindgen]
pub async fn lookup(document: &Document) {
    with_loading(async || {
        let membership_num_input =
            unwrap_or_alert(get_element_by_id_dyn(document, "membership-num-input"));
        let last_name_input = unwrap_or_alert(get_element_by_id_dyn(document, "last-name-input"));
        let first_name_input = unwrap_or_alert(get_element_by_id_dyn(document, "first-name-input"));
        let membership_num = get_value_from_element(&membership_num_input);
        let last_name = get_value_from_element(&last_name_input);
        let first_name = get_value_from_element(&first_name_input);

        let member_to_look_up = MemberToLookUp::new(
            if membership_num.is_empty() {
                None
            } else {
                Some(membership_num)
            },
            if last_name.is_empty() {
                None
            } else {
                Some(last_name)
            },
            if first_name.is_empty() {
                None
            } else {
                Some(first_name)
            },
        );

        let response = fetch(
            "/api/members/lookup",
            "post",
            Some("application/json"),
            Some(&json::to_string(&member_to_look_up)),
        )
        .await
        .map_err(|error| {
            Error::from_parent(
                "Une erreur s'est produite lors de la recherche. Veuillez réessayer.",
                error,
            )
        })?;

        let status = response.status();
        if (200..400).contains(&status) {
            let body = response
                .body()
                .clone()
                .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "No body"))?;
            let memberships = json::from_str::<Vec<Membership>>(&body);
            display_memberships(document, &memberships)?;
        } else if status == 400 {
            Err(Error::new(
                "Au moins un critère est nécessaire pour rechercher une adhésion.",
                "Missing criteria for looking memberships up.",
            ))?;
        } else if status == 401 {
            Err(Error::new(
                "Vous n'avez pas les droits pour rechercher une adhésion.",
                "Unauthorized to look memberships up.",
            ))?;
        } else {
            Err(Error::new(
                &format!(
                    "Une erreur s'est produite lors de la recherche d'adhésion [status: {status}]"
                ),
                &format!("Can't look memberships up [status: {status}]"),
            ))?;
        }

        toggle_go_to_email_step_button(document);
        next_step(document);
        Ok(())
    })
    .await;
}

fn display_memberships(document: &Document, memberships: &[Membership]) -> Result<()> {
    let memberships_container = get_element_by_id(document, "memberships")?;
    clear_element(&memberships_container);
    for membership in memberships {
        let card = create_known_membership_card(document, membership)?;
        append_child(&memberships_container, &card)?;
    }

    Ok(())
}
