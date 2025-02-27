use std::sync::Mutex;
use rocket::State;
use serde_json::json;
use crate::member::config::MembersProviderConfig;
use crate::member::download::download_members_list;
use crate::member::import_from_file::import_from_file;
use crate::web::api::members_state::MembersState;
use crate::tools::log_message_and_return;

#[get("/members")]
pub async fn list_members(members_state: &State<Mutex<MembersState>>) -> Result<String, String> {
    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while getting members."))?;
    let file_details = members_state.file_details();
    if let Some(details) = file_details {
        match import_from_file(details.filepath()) {
            Ok(members) => {
                members_state.set_members(members);
                Ok(json!(members_state.members()).to_string())
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
    let file_details = download_members_list(members_provider_config)
        .await
        .map_err(log_message_and_return("Can't download members list", "Can't download members list"))?;

    let mut members_state = members_state
        .lock()
        .map_err(log_message_and_return("Couldn't acquire lock", "Error while updating members."))?;
    members_state.set_file_details(file_details);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;
    use chrono::NaiveDate;
    use rocket::State;
    use crate::member::file_details::FileDetails;
    use crate::member::Member;
    use crate::web::api::members_state::MembersState;
    use crate::web::api::members_controller::list_members;
    use crate::tools::test::tests::temp_dir;

    const HEADER: &str = "Nom d'usage;Prénom;Sexe;Date de Naissance;Age;Numéro d'adhérent;Email;Réglé;Date Fin d'adhésion;Adherent expiré;Nom de structure;Code de structure";
    const MEMBER_AS_CSV: &str = "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oui;30-09-2025;Non;My club;Z01234";

    fn get_expected_member() -> Member {
        Member::new(
            "Doe".to_string(),
            "Jon".to_string(),
            "H".to_string(),
            NaiveDate::from_ymd_opt(1980, 2, 1),
            Some(45),
            "123456".to_string(),
            "email@address.com".to_string(),
            true,
            NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            false,
            "My club".to_string(),
            "Z01234".to_string(),
        )
    }


    // region list_members
    #[async_test]
    async fn should_list_members() {
        let temp_dir = temp_dir();
        let members_file_name = "members-2025-02-26.csv";
        let members_file_path = temp_dir.join(members_file_name);
        fs::write(&members_file_path, format!("{HEADER}\n{MEMBER_AS_CSV}")).unwrap();

        let file_details = FileDetails::new(NaiveDate::from_ymd_opt(2025, 2, 26).unwrap(), members_file_path.into_os_string());
        let members_state = MembersState::new(Some(file_details));
        let mutex = Mutex::new(members_state);
        let state = State::from(&mutex);

        let result: String = list_members(state).await.unwrap();
        let member: Member = serde_json::from_str(&result).unwrap();
        println!("{:?}", member);
        assert_eq!(get_expected_member(), member);
    }

    #[async_test]
    async fn should_not_list_members_when_no_file_details() {
        let members_state = MembersState::new(None);
        let mutex = Mutex::new(members_state);
        let state = State::from(&mutex);

        let error = list_members(state).await.err().unwrap();
        assert!(error.contains("Can't find file."));
    }

    #[async_test]
    async fn should_not_list_members_when_no_file() {
        let temp_dir = temp_dir();
        let members_file_name = "members-2025-02-26.csv";
        let members_file_path = temp_dir.join(members_file_name);

        let file_details = FileDetails::new(NaiveDate::from_ymd_opt(2025, 2, 26).unwrap(), members_file_path.into_os_string());
        let members_state = MembersState::new(Some(file_details));
        let mutex = Mutex::new(members_state);
        let state = State::from(&mutex);

        let result: String = list_members(state).await.err().unwrap();
        assert!(result.contains("CantOpenMembersFile"));
    }
    // endregion
}
