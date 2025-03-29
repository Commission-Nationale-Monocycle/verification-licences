use crate::check_memberships::toggle_go_to_email_step_button;
use crate::component::stepper::next_step;
use crate::error::{DEFAULT_SERVER_ERROR_MESSAGE, Error};
use crate::fileo::load_members_from_csv;
use crate::user_interface::with_loading;
use crate::utils::get_document;
use crate::web::fetch;
use crate::{json, user_interface};
use dto::checked_member::CheckedMember;
use dto::csv_member::CsvMember;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn handle_members_to_check_file() {
    with_loading(async || {
        let document = get_document()?;

        let (members_to_check, wrong_lines) =
            load_members_from_csv::load_members_to_check(&document).await?;

        user_interface::render_lines(&document, &members_to_check, &wrong_lines)
    })
    .await;
}

#[wasm_bindgen]
pub async fn handle_form_submission() {
    with_loading(async || {
        let document = &get_document()?;

        let (members_to_check, _) = load_members_from_csv::load_members_to_check(document).await?;
        if members_to_check.is_empty() {
            return Err(Error::new(
                "Impossible de valider un fichier vide, ou dont aucune ligne n'est valide. Veuillez sélectionner un autre fichier..",
                "Can't check an empty file.",
            ));
        }

        let url = "/api/members/csv/check";
        let body = json::to_string(&members_to_check);
        let response = fetch(
            url,
            "post",
            Some("application/json"),
            Some(&body),
        )
            .await
            .map_err(|error| Error::from_parent(DEFAULT_SERVER_ERROR_MESSAGE, error))?;

        let status = response.status();
        if (200..400).contains(&status) {
            let text = response.body().clone().unwrap_or(String::new());
            let checked_members: Vec<CheckedMember<CsvMember>> = json::from_str(&text);
            user_interface::handle_checked_members(document, &checked_members)?;
            toggle_go_to_email_step_button(document);
            next_step(document);

            Ok(())
        } else {
            Err(Error::from_server_status_error(status))
        }
    })
        .await;
}
