use crate::Result;
use crate::card_creator::create_card_for_member_to_check;
use crate::component::accordion::{AccordionElement, create_accordion};
use crate::component::stepper::next_step;
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::json;
use crate::uda::credentials::UdaCredentials;
use crate::user_interface::with_loading;
use crate::utils::{
    add_class, append_child, clear_element, get_element_by_id, get_element_by_id_dyn,
    get_value_from_element,
};
use crate::web::fetch;
use dto::uda_member::UdaMember;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element, HtmlElement, HtmlSelectElement};

#[wasm_bindgen(js_name = "importFromUda")]
pub async fn import_from_uda_page(document: &Document) {
    with_loading(async || {
        let is_logged_in = login(document)
            .await
            .map_err(Error::from_parent_with_default_message)?;
        if !is_logged_in {
            return Err(Error::new(
                "Vos identifiants sont incorrects. Veuillez réessayer.",
                "Wrong credentials.",
            ));
        }
        let members = retrieve_members(document).await?;

        display_members(document, &members).map_err(Error::from_parent_with_default_message)?;
        next_step(document);

        Ok(())
    })
    .await;
}

async fn login(document: &Document) -> Result<bool> {
    let select = get_element_by_id_dyn::<HtmlSelectElement>(document, "uda-instance-selector")?;
    let login_input = get_element_by_id_dyn(document, "login")?;
    let password_input = get_element_by_id_dyn(document, "password")?;
    let instance = select.value();
    let login = get_value_from_element(&login_input);
    let password = get_value_from_element(&password_input);

    #[derive(Serialize)]
    struct Credentials {
        credentials: UdaCredentials,
    }
    let credentials = Credentials {
        credentials: UdaCredentials::new(instance, login, password),
    };
    let body = json::to_string(&credentials);

    let response = fetch(
        "/api/uda/login",
        "post",
        Some("application/json"),
        Some(body.as_str()),
    )
    .await?;

    let status = response.status();
    if (200..400).contains(&status) {
        Ok(true)
    } else if status == 401 {
        Ok(false)
    } else {
        Err(Error::new(
            "Impossible de se connecter à UDA. Veuillez réessayer.",
            &format!("Can't login [status: {status}"),
        ))
    }
}

async fn retrieve_members(document: &Document) -> Result<Vec<UdaMember>> {
    let response = fetch("/api/uda/retrieve", "get", None, None).await?;
    let status = response.status();
    if (200..400).contains(&status) {
        let body = response
            .body()
            .clone()
            .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "No body"))?;
        get_element_by_id(document, "members-as-json")?.set_text_content(Some(&body));
        let members = json::from_str(&body);
        Ok(members)
    } else if status == 401 {
        Err(Error::new(
            "Vous n'avez pas les droits pour récupérer les participants depuis l'instance UDA sélectionnée.",
            "Unauthorized to retrieve UDA members.",
        ))
    } else {
        Err(Error::new(
            "Impossible de récupérer les membres depuis UDA. Veuillez réessayer.",
            &format!("Unable to retrieve UDA members [status: {status}]"),
        ))
    }
}

fn display_members(document: &Document, members: &Vec<UdaMember>) -> Result<()> {
    let container = get_element_by_id(document, "members")?;
    clear_element(&container);

    let mut confirmed_member_cards = vec![];
    let mut not_confirmed_member_cards = vec![];

    for member in members {
        let card = create_card_for_member_to_check(document, member)?;
        if *member.confirmed() {
            confirmed_member_cards.push(card);
        } else {
            not_confirmed_member_cards.push(card);
        }
    }

    let element = create_accordion_for_members_to_check(
        document,
        &confirmed_member_cards,
        &not_confirmed_member_cards,
    )?;
    append_child(&container, &element)?;

    Ok(())
}

fn create_accordion_for_members_to_check(
    document: &Document,
    confirmed_member_cards: &[Element],
    not_confirmed_member_cards: &[Element],
) -> Result<HtmlElement> {
    let not_confirmed_element = create_accordion_line_for_members_to_check(
        document,
        "not-confirmed",
        "Membres non-confirmés",
        not_confirmed_member_cards,
        true,
    )?;
    let confirmed_element = create_accordion_line_for_members_to_check(
        document,
        "confirmed",
        "Membres confirmés",
        confirmed_member_cards,
        false,
    )?;

    let elements = [not_confirmed_element, confirmed_element]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    create_accordion(document, &elements, true)
}

fn create_accordion_line_for_members_to_check(
    document: &Document,
    id: &str,
    title: &str,
    cards: &[Element],
    active: bool,
) -> Result<Option<AccordionElement>> {
    if !cards.is_empty() {
        let body_container = document.create_element("div")?;
        add_class(&body_container, "members");
        for card in cards {
            append_child(&body_container, card)?;
        }

        let title_container = document.create_element("div")?;
        title_container.set_inner_html(title);

        Ok(Some(AccordionElement::new(
            id.to_owned(),
            title_container,
            body_container,
            active,
        )))
    } else {
        Ok(None)
    }
}
