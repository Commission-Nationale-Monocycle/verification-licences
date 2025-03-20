use crate::alert::unwrap_or_alert;
use crate::card_creator::create_card_for_uda_checked_member;
use crate::error::Error;
use crate::stepper::next_step;
use crate::user_interface::set_loading;
use crate::utils::{append_child, clear_element, get_element_by_id};
use crate::web::fetch;
use crate::{Result, json};
use dto::checked_member::CheckedMember;
use dto::uda_member::UdaMember;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Document;

#[wasm_bindgen]
pub async fn check_members(document: &Document) {
    unwrap_or_alert(set_loading(true));
    let checked_members = unwrap_or_alert(check(document).await);
    unwrap_or_alert(handle_checked_members(document, &checked_members));
    next_step(document);
    unwrap_or_alert(set_loading(false));
}

fn handle_checked_members(
    document: &Document,
    checked_members: &Vec<CheckedMember<UdaMember>>,
) -> Result<()> {
    let parent = get_element_by_id(document, "checked-members")?;
    clear_element(&parent);
    for checked_member in checked_members {
        let card = create_card_for_uda_checked_member(document, checked_member)?;
        append_child(&parent, &card)?;
    }

    Ok(())
}

async fn check(document: &Document) -> Result<Vec<CheckedMember<UdaMember>>> {
    let members = get_element_by_id(document, "members-as-json")?
        .text_content()
        .ok_or_else(|| {
            Error::new("Liste de membres à vérifier introuvable. Veuillez réessayer.".to_owned())
        })?;
    let response = fetch(
        "/api/members/uda/check",
        "post",
        Some("application/json"),
        Some(members.as_str()),
    )
    .await
    .map_err(|error| {
        Error::from_parent(
            "Une erreur s'est produite lors de la vérification des participants.".to_owned(),
            Error::new(error.to_string()),
        )
    })?;

    let status = response.status();
    if (200..400).contains(&status) {
        let body = response
            .body()
            .clone()
            .ok_or_else(|| Error::new("No body".to_owned()))?;
        let checked_members = json::from_str(&body);
        Ok(checked_members)
    } else if status == 401 {
        Err(Error::new(
            "Vous n'avez pas les droits pour vérifier les participants.".to_owned(),
        ))
    } else {
        Err(Error::new(format!(
            "Une erreur s'est produite lors de la vérification des participants [status: {status}]"
        )))
    }
}
