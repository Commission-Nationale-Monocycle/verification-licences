use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::user_interface::set_loading;
use crate::utils::{get_document, get_element_by_id};
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Date;

/// Validate the field, then try to log into Fileo app.
/// If it succeeds, then redirect to the verification page.
#[wasm_bindgen]
pub async fn update_fileo_list() {
    unwrap_or_alert(set_loading(true));

    let url = "/api/fileo/memberships";
    match fetch(url, "get", None, None).await {
        Ok(response) => {
            let status = response.status();
            if (200..400).contains(&status) {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Mise à jour effectuée. Vous pouvez désormais vérifier les licences.",
                    AlertLevel::Info,
                );
                let document = unwrap_without_alert(get_document());
                let last_update_field =
                    unwrap_or_alert(get_element_by_id(&document, "last-update"));
                let now = Date::new_0();
                let day = now.get_date();
                let month = now.get_month() + 1;
                let year = now.get_full_year();
                last_update_field
                    .set_text_content(Some(&format!("{:02}/{:02}/{}", day, month, year)))
            } else if status == 401 {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Vos identifiants sont incorrects. Veuillez réessayer.",
                    AlertLevel::Error,
                );
            } else {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Impossible de mettre à jour la liste. Veuillez réessayer.",
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
