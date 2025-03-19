use crate::Result;
use crate::alert::{AlertLevel, create_alert, unwrap_or_alert};
use crate::error::Error;
use crate::json;
use crate::stepper::next_step;
use crate::template::get_template;
use crate::uda::credentials::UdaCredentials;
use crate::user_interface::set_loading;
use crate::utils::{
    add_class, append_child, get_element_by_id, get_element_by_id_dyn, get_value_from_element,
    query_selector_single_element, set_attribute,
};
use crate::web::fetch;
use dto::uda_member::UdaMember;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element, HtmlSelectElement};

#[wasm_bindgen(js_name = "importFromUda")]
pub async fn import_from_uda_page(document: &Document) {
    unwrap_or_alert(set_loading(true));

    let is_logged_in = unwrap_or_alert(login(document).await.map_err(|error| {
        Error::from_parent(
            "Erreur, veuillez réessyer.".to_owned(),
            Error::new(error.to_string()),
        )
    }));
    if !is_logged_in {
        create_alert(
            "Vos identifiants sont incorrects. Veuillez réessayer.",
            AlertLevel::Error,
        );
        return;
    }
    let members = unwrap_or_alert(retrieve_members().await.map_err(|error| {
        Error::from_parent(
            "Erreur, veuillez réessyer.".to_owned(),
            Error::new(error.to_string()),
        )
    }));

    unwrap_or_alert(display_members(document, &members).map_err(|error| {
        Error::from_parent(
            "Erreur, veuillez réessyer.".to_owned(),
            Error::new(error.to_string()),
        )
    }));
    next_step(document);

    unwrap_or_alert(set_loading(false));
}

async fn login(document: &Document) -> Result<bool> {
    let select = get_element_by_id_dyn::<HtmlSelectElement>(document, "uda-instance-selector")?;
    let login_input = get_element_by_id_dyn(document, "login")?;
    let password_input = get_element_by_id_dyn(document, "password")?;
    let instance = select.value();
    let login = get_value_from_element(&login_input);
    let password = get_value_from_element(&password_input);

    let credentials = UdaCredentials::new(instance, login, password);
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
        Err(Error::new(format!("Can't login [status: {status}")))
    }
}

async fn retrieve_members() -> Result<Vec<UdaMember>> {
    let response = fetch("/api/uda/retrieve", "get", None, None).await?;
    let status = response.status();
    if (200..400).contains(&status) {
        let body = response
            .body()
            .clone()
            .ok_or_else(|| Error::new("No body".to_owned()))?;
        let members = json::from_str(&body);
        Ok(members)
    } else if status == 401 {
        // FIXME: this is never displayed, as another later alert would hide this first one.
        create_alert(
            "Vous n'avez pas les droits pour récupérer les participants depuis l'instance UDA sélectionnée.",
            AlertLevel::Error,
        );
        Err(Error::new("Can't retrieve UDA members".to_owned()))
    } else {
        Err(Error::new(format!(
            "Can't retrieve UDA members [status: {status}"
        )))
    }
}

fn display_members(document: &Document, members: &Vec<UdaMember>) -> Result<()> {
    let container = get_element_by_id(document, "checked-members")?;
    for member in members {
        match create_member_card(document, member) {
            Ok(element) => {
                append_child(&container, &element)?;
            }
            Err(error) => {
                log::warn!("UDA member can't be displayed: {:?}", error);
            }
        }
    }

    Ok(())
}

fn create_member_card(document: &Document, member: &UdaMember) -> Result<Element> {
    let element = get_template(document, "member-template")?;
    query_selector_single_element(&element, ".membership-number")?.set_inner_html(
        &member
            .membership_number()
            .clone()
            .unwrap_or("Non renseigné".to_string()),
    );
    query_selector_single_element(&element, ".name")?.set_inner_html(member.last_name().as_str());
    query_selector_single_element(&element, ".first-name")?
        .set_inner_html(member.first_name().as_str());
    let club_element = query_selector_single_element(&element, ".club")?;
    if let Some(club) = member.club() {
        club_element.set_inner_html(club);
    } else {
        let club_parent = club_element
            .parent_element()
            .ok_or_else(|| Error::new("No club element parent.".to_owned()))?;
        add_class(&club_parent, "hidden");
    }
    let email_address_element = query_selector_single_element(&element, ".email-address")?;
    let email_address = member.email().as_str();
    email_address_element.set_inner_html(email_address);
    set_attribute(
        &email_address_element,
        "href",
        &format!("mailto:{email_address}"),
    )?;

    Ok(element)
}
