use std::sync::Mutex;
use rocket::State;
use crate::member::download::download_members_list;
use crate::member::file_details::FileDetails;
use crate::member::import_from_file::import_from_file;
use crate::server::members_state::MembersState;

#[get("/members")]
pub async fn members(members_state: &State<Mutex<MembersState>>) -> Result<String, String> {
    let mut members_state = members_state.lock().unwrap();
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
pub async fn update_members(members_state: &State<Mutex<MembersState>>) {
    let (datetime, filename) = match download_members_list().await {
        Ok((date, filename)) => { (date, filename) }
        Err(_) => { panic!("Oops") }
    };

    let mut members_state = members_state.lock().unwrap();
    members_state.set_file_details(FileDetails::new(datetime, filename));
}