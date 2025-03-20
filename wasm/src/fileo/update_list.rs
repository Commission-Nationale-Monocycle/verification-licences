use crate::alert::{AlertLevel, create_alert};
use crate::error::Error;
use crate::user_interface::with_loading;
use crate::utils::{get_document, get_element_by_id};
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Date;

/// Validate the field, then try to log into Fileo app.
/// If it succeeds, then redirect to the verification page.
#[wasm_bindgen]
pub async fn update_fileo_list() {
    with_loading(async || {
        let url = "/api/fileo/memberships";
        let response = fetch(url, "get", None, None).await.map_err(|error| {
            Error::from_parent(
                "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
                error,
            )
        })?;
        let status = response.status();
        if (200..400).contains(&status) {
            create_alert(
                "Mise à jour effectuée. Vous pouvez désormais vérifier les licences.",
                AlertLevel::Info,
            );
            let document = get_document()?;
            let last_update_field = get_element_by_id(&document, "last-update")?;
            let now = Date::new_0();
            let day = now.get_date();
            let month = now.get_month() + 1;
            let year = now.get_full_year();
            last_update_field.set_text_content(Some(&format!("{:02}/{:02}/{}", day, month, year)));

            Ok(())
        } else if status == 401 {
            Err(Error::new(
                "Vos identifiants sont incorrects. Veuillez réessayer.",
                "Wrong credentials",
            ))
        } else {
            Err(Error::new(
                "Impossible de mettre à jour la liste. Veuillez réessayer.",
                &format!("Server error: {}", status),
            ))
        }
    })
    .await;
}
