use crate::error::{DEFAULT_SERVER_ERROR_MESSAGE, Error};
use crate::user_interface::with_loading;
use crate::utils::get_window;
use crate::web::fetch;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn update_uda_instances_list() {
    with_loading(async || {
        let url = "/api/uda/instances";
        let response = fetch(url, "get", None, None)
            .await
            .map_err(|error| Error::from_parent(DEFAULT_SERVER_ERROR_MESSAGE, error))?;
        let status = response.status();
        if (200..400).contains(&status) {
            let location = get_window()?.location();
            location.reload().map_err(|error| {
                Error::from_parent(
                    &format!("Impossible de recharger la page : {error:?}"),
                    Error::from(error),
                )
            })?;

            Ok(())
        } else {
            Err(Error::from_server_status_error(status))
        }
    })
    .await;
}
