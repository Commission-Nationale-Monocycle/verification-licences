use crate::Result;
use crate::alert::{AlertLevel, create_alert, unwrap_or_alert};
use crate::card_creator::create_card_for_uda_member_to_check;
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::json;
use crate::stepper::next_step;
use crate::uda::credentials::UdaCredentials;
use crate::user_interface::set_loading;
use crate::utils::{
    append_child, clear_element, get_element_by_id, get_element_by_id_dyn, get_value_from_element,
};
use crate::web::fetch;
use dto::uda_member::UdaMember;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, HtmlSelectElement};

#[wasm_bindgen(js_name = "importFromUda")]
pub async fn import_from_uda_page(document: &Document) {
    unwrap_or_alert(set_loading(true));

    let is_logged_in = unwrap_or_alert(
        login(document)
            .await
            .map_err(|error| Error::from_parent(DEFAULT_ERROR_MESSAGE.to_owned(), error)),
    );
    if !is_logged_in {
        create_alert(
            "Vos identifiants sont incorrects. Veuillez réessayer.",
            AlertLevel::Error,
        );
        return;
    }
    let members = unwrap_or_alert(retrieve_members(document).await);

    unwrap_or_alert(
        display_members(document, &members)
            .map_err(|error| Error::from_parent(DEFAULT_ERROR_MESSAGE.to_owned(), error)),
    );
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
        Err(Error::new(
            "Impossible de se connecter à UDA. Veuillez réessayer.".to_owned(),
            format!("Can't login [status: {status}"),
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
            .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE.to_owned(), "No body".to_owned()))?;
        get_element_by_id(document, "members-as-json")?.set_text_content(Some(&body));
        let members = json::from_str(&body);
        Ok(members)
    } else if status == 401 {
        Err(Error::new("Vous n'avez pas les droits pour récupérer les participants depuis l'instance UDA sélectionnée.".to_owned(),
        "Unauthorized to retrieve UDA members.".to_owned()))
    } else {
        Err(Error::new(
            "Impossible de récupérer les membres depuis UDA. Veuillez réessayer.".to_owned(),
            format!("Unable to retrieve UDA members [status: {status}]"),
        ))
    }
}

fn display_members(document: &Document, members: &Vec<UdaMember>) -> Result<()> {
    let container = get_element_by_id(document, "members")?;
    clear_element(&container);
    for member in members {
        match create_card_for_uda_member_to_check(document, member) {
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
