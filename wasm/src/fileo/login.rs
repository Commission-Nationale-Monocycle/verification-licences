use crate::error::Error;
use crate::fileo::credentials::FileoCredentials;
use crate::json;
use crate::user_interface::with_loading;
use crate::utils::{get_document, get_element_by_id_dyn, get_location, get_value_from_element};
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlInputElement, UrlSearchParams};

/// Validate the field, then try to log into Fileo app.
/// If it succeeds, then redirect to the verification page.
#[wasm_bindgen(js_name = "logIntoFileo")]
pub async fn login() {
    with_loading(async || {
        let document = get_document()?;
        let login_field = get_element_by_id_dyn::<HtmlInputElement>(&document, "login")?;
        let password_field = get_element_by_id_dyn::<HtmlInputElement>(&document, "password")?;

        if !login_field.report_validity() || !password_field.report_validity() {
            return Ok(());
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
                    let location = get_location()?;
                    let query_params = location.search().map_err(Error::from)?;
                    let query_params =
                        UrlSearchParams::new_with_str(&query_params).map_err(Error::from)?;
                    let url_to_redirect = if let Some(redirect) = query_params.get("page") {
                        redirect
                    } else {
                        "/csv/check".to_owned()
                    };
                    let result = location.set_href(&url_to_redirect);
                    if let Err(error) = result {
                        Err(Error::from_parent(
                            "Erreur lors de la redirection. Veuillez actualiser la page.",
                            Error::from(error),
                        ))
                    } else {
                        Ok(())
                    }
                } else if status == 401 {
                    Err(Error::new(
                        "Vos identifiants sont incorrects. Veuillez réessayer.",
                        "Wrong credentials, can't login",
                    ))
                } else {
                    Err(Error::new(
                        "Impossible de se connecter. Veuillez réessayer.",
                        &format!("Server error: {}", status),
                    ))
                }
            }
            Err(error) => Err(Error::from_parent(
                "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
                error,
            )),
        }
    })
    .await;
}
