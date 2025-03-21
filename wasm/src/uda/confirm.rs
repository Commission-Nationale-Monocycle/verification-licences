use crate::component::alert::{AlertLevel, create_alert};
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::json::to_string;
use crate::user_interface::with_loading;
use crate::utils::{get_body, get_value_from_element, query_selector_all};
use crate::web::fetch;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn confirm_members() {
    with_loading(async || {
        let body = get_body()?;
        let id_inputs =
            query_selector_all(&body, ".checked-member:has(.membership-up-to-date) .uda-id")?;

        let mut ids = vec![];
        for id_input in id_inputs {
            let id = get_value_from_element(&id_input.dyn_into()?);
            let id = id
                .parse::<u16>()
                .map_err(|error| Error::new(DEFAULT_ERROR_MESSAGE, error.to_string().as_str()))?;
            ids.push(id);
        }

        let body = to_string(&ids);
        let response = fetch(
            "/api/uda/confirm",
            "post",
            Some("application/json"),
            Some(&body),
        )
        .await?;

        let status = response.status();
        if (200..400).contains(&status) {
            let message = match ids.len() {
                0 => "Aucun membre n'a été confirmé sur UDA.".to_owned(),
                1 => "Un membre a été confirmé sur UDA.".to_owned(),
                n => format!("{n} membres ont été confirmés sur UDA."),
            };
            create_alert(&message, AlertLevel::Info);
        } else {
            Err(Error::from_server_status_error(status))?;
        }

        Ok(())
    })
    .await;
}
