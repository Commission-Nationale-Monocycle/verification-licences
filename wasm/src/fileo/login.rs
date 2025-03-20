use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::error::Error;
use crate::fileo::credentials::FileoCredentials;
use crate::json;
use crate::user_interface::set_loading;
use crate::utils::{get_document, get_element_by_id_dyn, get_location, get_value_from_element};
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlInputElement, UrlSearchParams};

/// Validate the field, then try to log into Fileo app.
/// If it succeeds, then redirect to the verification page.
#[wasm_bindgen(js_name = "logIntoFileo")]
pub async fn login() {
    unwrap_or_alert(set_loading(true));

    let document = unwrap_without_alert(get_document());
    let login_field = unwrap_without_alert(get_element_by_id_dyn::<HtmlInputElement>(
        &document, "login",
    ));
    let password_field = unwrap_without_alert(get_element_by_id_dyn::<HtmlInputElement>(
        &document, "password",
    ));

    if !login_field.report_validity() || !password_field.report_validity() {
        unwrap_or_alert(set_loading(false));
        return;
    }

    let login = get_value_from_element(&login_field);
    let password = get_value_from_element(&password_field);
    let url = "/api/fileo/login";

    let credentials = FileoCredentials::new(login, password);
    let body = json::to_string(&credentials);

    match fetch(url, "post", Some("application/json"), Some(&body)).await {
        Ok(response) => {
            let status = response.status();
            if (200..400).contains(&status) {
                unwrap_or_alert(set_loading(false));
                let location = unwrap_or_alert(get_location());
                let query_params = unwrap_or_alert(location.search().map_err(Error::from));
                let query_params = unwrap_or_alert(
                    UrlSearchParams::new_with_str(&query_params).map_err(Error::from),
                );
                let url_to_redirect = if let Some(redirect) = query_params.get("page") {
                    redirect
                } else {
                    "/csv/check".to_owned()
                };
                let result = location.set_href(&url_to_redirect);
                if let Err(error) = result {
                    create_alert(
                        "Erreur lors de la redirection. Veuillez actualiser la page.",
                        AlertLevel::Error,
                    );
                    log::error!("Can't redirect user: {error:?}");
                }
            } else if status == 401 {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Vos identifiants sont incorrects. Veuillez réessayer.",
                    AlertLevel::Error,
                );
            } else {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Impossible de se connecter. Veuillez réessayer.",
                    AlertLevel::Error,
                );
                log::error!("Server error: {}", status);
            }
        }
        Err(error) => {
            unwrap_or_alert(set_loading(false));
            create_alert(
                "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
                AlertLevel::Error,
            );
            log::error!("Server error: {:?}", error);
        }
    }
}
