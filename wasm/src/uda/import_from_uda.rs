use crate::Result;
use crate::alert::{AlertLevel, create_alert, unwrap_or_alert};
use crate::error::Error;
use crate::json;
use crate::stepper::next_step;
use crate::uda::credentials::UdaCredentials;
use crate::user_interface::set_loading;
use crate::utils::{get_element_by_id_dyn, get_value_from_element};
use crate::web::fetch;
use dto::uda::Participant;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, HtmlSelectElement};

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
    // FIXME: use participants
    let participants = unwrap_or_alert(retrieve_participants().await.map_err(|error| {
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

async fn retrieve_participants() -> Result<Vec<Participant>> {
    let response = fetch("/api/uda/retrieve", "get", None, None).await?;
    let status = response.status();
    if (200..400).contains(&status) {
        let body = response
            .body()
            .clone()
            .ok_or_else(|| Error::new("No body".to_owned()))?;
        let participants = json::from_str(&body);
        Ok(participants)
    } else if status == 401 {
        // FIXME: this is never displayed, as another later alert would hide this first one.
        create_alert(
            "Vous n'avez pas les droits pour récupérer les participants depuis l'instance UDA sélectionnée.",
            AlertLevel::Error,
        );
        Err(Error::new("Can't retrieve participants".to_owned()))
    } else {
        Err(Error::new(format!(
            "Can't retrieve participants [status: {status}"
        )))
    }
}
