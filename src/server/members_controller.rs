use std::env;
use std::sync::Mutex;
use rocket::State;
use crate::member::config::MembersProviderConfig;
use crate::member::download::download_members_list;
use crate::member::import_from_file::import_from_file;
use crate::server::members_state::MembersState;
use crate::tools::log_message_and_return;

#[get("/members")]
pub async fn members(members_state: &State<Mutex<MembersState>>) -> Result<String, String> {
    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while getting members."))?;
    let file_details = members_state.file_details();
    if let Some(details) = file_details {
        match import_from_file(details.filename()) {
            Ok(members) => {
                members_state.set_members(members);
                Ok(format!("{:#?}", members_state.members()))
            }
            Err(e) => {
                Err(format!("{e:?}"))
            }
        }
    } else {
        Err("Can't find file.".to_string())
    }
}

#[post("/members")]
pub async fn update_members(
    members_provider_config: &State<MembersProviderConfig>,
    members_state: &State<Mutex<MembersState>>,
) -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let file_details = download_members_list(&args, members_provider_config)
        .await
        .map_err(log_message_and_return("Can't download members list", "Can't download members list"))?;

    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while updating members."))?;
    members_state.set_file_details(file_details);

    Ok(())
}