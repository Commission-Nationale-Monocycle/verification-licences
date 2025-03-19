use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::error::Error;
use crate::user_interface::set_loading;
use crate::utils::get_window;
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn update_uda_instances_list() {
    unwrap_or_alert(set_loading(true));

    let url = "/api/uda/instances";
    match fetch(url, "get", None, None).await {
        Ok(response) => {
            let status = response.status();
            if (200..400).contains(&status) {
                let location = unwrap_without_alert(get_window()).location();
                unwrap_or_alert(
                    location
                        .reload()
                        .map_err(|error| Error::new(format!("Can't reload page: {error:?}"))),
                );
                unwrap_or_alert(set_loading(false));
            } else {
                unwrap_or_alert(set_loading(false));
                create_alert(
                    "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.",
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
