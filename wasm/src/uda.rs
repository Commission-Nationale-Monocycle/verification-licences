use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::build_client;
use crate::error::Error;
use crate::user_interface::set_loading;
use crate::utils::get_window;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn update_uda_instances_list() {
    unwrap_or_alert(set_loading(true));

    let client = build_client();

    let window = unwrap_without_alert(get_window());
    let location = window.location();
    let origin = location.origin().unwrap();
    let url = format!("{origin}/api/uda/instances");
    let response = client.get(&url).send().await.unwrap_or_else(|error| {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Impossible d'envoyer la requête. Veuillez réessayer.",
            AlertLevel::Error,
        );
        panic!("can't send request: {error:?}")
    });

    let status = response.status();
    if status.is_success() || status.is_redirection() {
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
        log::error!("Server error: {}", response.status().as_str())
    }
}
