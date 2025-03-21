use crate::card_creator::create_card_for_checked_member;
use crate::component::accordion::{AccordionElement, create_accordion};
use crate::component::stepper::next_step;
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::user_interface::with_loading;
use crate::utils::{add_class, append_child, clear_element, get_element_by_id};
use crate::web::fetch;
use crate::{Result, json};
use dto::checked_member::{CheckedMember, MemberStatus};
use dto::uda_member::UdaMember;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element, HtmlElement};

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

    let mut up_to_date_member_cards = vec![];
    let mut expired_member_cards = vec![];
    let mut unknown_member_cards = vec![];
    for checked_member in checked_members {
        let card = create_card_for_checked_member(document, checked_member)?;
        match checked_member.compute_member_status() {
            MemberStatus::UpToDate => up_to_date_member_cards.push(card),
            MemberStatus::Expired => expired_member_cards.push(card),
            MemberStatus::Unknown => unknown_member_cards.push(card),
        }
    }
    let accordion = create_accordion_for_checked_members(
        document,
        &up_to_date_member_cards,
        &expired_member_cards,
        &unknown_member_cards,
    )?;
    append_child(&parent, &accordion)?;

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

fn create_accordion_for_checked_members(
    document: &Document,
    up_to_date_member_cards: &[Element],
    expired_member_cards: &[Element],
    unknown_member_cards: &[Element],
) -> Result<HtmlElement> {
    let up_to_date_element = create_accordion_line_for_checked_members(
        document,
        "up-to-date",
        "Membres à jour",
        up_to_date_member_cards,
    )?;
    let expired_element = create_accordion_line_for_checked_members(
        document,
        "expired",
        "Membres expirés",
        expired_member_cards,
    )?;
    let unknown_element = create_accordion_line_for_checked_members(
        document,
        "unknown",
        "Membres inconnus",
        unknown_member_cards,
    )?;

    let elements = [up_to_date_element, expired_element, unknown_element]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    create_accordion(document, &elements, true)
}

fn create_accordion_line_for_checked_members(
    document: &Document,
    id: &str,
    title: &str,
    cards: &[Element],
) -> Result<Option<AccordionElement>> {
    if !cards.is_empty() {
        let body_container = document.create_element("div")?;
        add_class(&body_container, "checked-members");
        for card in cards {
            append_child(&body_container, card)?;
        }

        let title_container = document.create_element("div")?;
        title_container.set_inner_html(title);

        Ok(Some(AccordionElement::new(
            id.to_owned(),
            title_container,
            body_container,
            true,
        )))
    } else {
        Ok(None)
    }
}
