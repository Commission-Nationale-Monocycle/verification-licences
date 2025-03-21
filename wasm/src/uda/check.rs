use crate::card_creator::create_card_for_uda_checked_member;
use crate::component::stepper::next_step;
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::user_interface::with_loading;
use crate::utils::{append_child, clear_element, get_element_by_id};
use crate::web::fetch;
use crate::{Result, json};
use dto::checked_member::CheckedMember;
use dto::uda_member::UdaMember;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Document;

#[wasm_bindgen]
pub async fn check_members(document: &Document) {
    with_loading(async || {
        let checked_members = check(document).await?;
        handle_checked_members(document, &checked_members)?;
        next_step(document);
        Ok(())
    })
    .await;
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
    let element_id = "members-as-json";
    let members = get_element_by_id(document, element_id)?
        .text_content()
        .ok_or_else(|| {
            Error::new(
                "Liste de membres à vérifier introuvable. Veuillez réessayer.",
                &format!("No members to check [id: {element_id}]."),
            )
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
            "Une erreur s'est produite lors de la vérification des participants.",
            error,
        )
    })?;

    let status = response.status();
    if (200..400).contains(&status) {
        let body = response
            .body()
            .clone()
            .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "No body"))?;
        let checked_members = json::from_str(&body);
        Ok(checked_members)
    } else if status == 401 {
        Err(Error::new(
            "Vous n'avez pas les droits pour vérifier les participants.",
            "Unauthorized to check participants.",
        ))
    } else {
        Err(Error::new(
            &format!(
                "Une erreur s'est produite lors de la vérification des participants [status: {status}]"
            ),
            &format!("Can't check participants [status: {status}]"),
        ))
    }
}
