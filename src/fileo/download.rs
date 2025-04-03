use std::ffi::{OsStr, OsString};
use std::io::BufReader;
use std::path::PathBuf;

use crate::error::{ApplicationError, Result};
use crate::fileo::credentials::FileoCredentials;
use crate::fileo::error::FileoError;
use crate::fileo::error::FileoError::{
    CantLoadListOnServer, CantRetrieveDownloadLink, MalformedMembershipsDownloadResponse,
    MembershipsFileFolderCreationFailed, MembershipsFileWriteFailed, NoDownloadLink,
};
use crate::fileo::imported_membership::ImportedMembership;
use crate::membership::config::MembershipsProviderConfig;
use crate::membership::file_details::FileDetails;
use crate::tools::web::build_client;
use crate::tools::{log_error_and_return, log_message, log_message_and_return};
use crate::web::error::WebError::{
    CantReadPageContent, ConnectionFailed, LackOfPermissions, NotFound, WrongCredentials,
};
use chrono::Local;
use csv::Reader;
use dto::membership::Membership;
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use log::{debug, error};
use regex::Regex;
use reqwest::{Client, RequestBuilder};
use rocket::form::validate::Contains;
use rocket::http::ContentType;

/// Download the memberships list from Fileo and write it to the disk.
/// Require valid credentials to log in to Fileo.
pub async fn download_memberships_list(
    memberships_provider_config: &MembershipsProviderConfig,
    credentials: &FileoCredentials,
) -> Result<Vec<Membership>> {
    let folder = memberships_provider_config.folder();
    let host = memberships_provider_config.host();
    let download_link_regex = memberships_provider_config.download_link_regex();
    create_memberships_file_dir(folder)?;

    let client = build_client()?;
    login_to_fileo(&client, host, credentials).await?;
    load_list_into_server_session(&client, host).await?;
    let download_url = retrieve_download_link(&client, host, download_link_regex).await?;
    let file_content = download_list(&client, &download_url).await?;
    Ok(parse_file(&file_content))
}

// region Requests
pub async fn login_to_fileo(
    client: &Client,
    domain: &str,
    credentials: &FileoCredentials,
) -> Result<()> {
    let request = prepare_request_for_connection(client, domain, credentials);
    let response = request.send().await.map_err(log_message_and_return(
        "Connection failed...",
        ConnectionFailed,
    ))?;
    let status = response.status();
    if !status.is_success() {
        error!("Connection failed because of status {status}...");
        return Err(ApplicationError::from(ConnectionFailed));
    }

    let text = response.text().await.map_err(log_message_and_return(
        "Couldn't get text of response",
        ConnectionFailed,
    ))?;

    if text.contains("L'identifiant et le mot de passe ne correspondent pas")
        || text.contains("Le champ 'Identifiant' est obligatoire")
        || text.contains("Le champ 'Mot de passe' est obligatoire")
    {
        Err(ApplicationError::from(WrongCredentials))
    } else if !text
        .contains("Profil Commission thématique - UNSLL - Commission Nationale Monocycle")
    {
        Err(ApplicationError::from(LackOfPermissions))
    } else {
        Ok(())
    }
}

async fn load_list_into_server_session(client: &Client, domain: &str) -> Result<()> {
    let request = prepare_request_for_loading_list_into_server_session(client, domain);
    let response = request.send().await.map_err(log_message_and_return(
        "The server couldn't load the list.",
        CantLoadListOnServer,
    ))?;
    let status = response.status();
    if status.is_success() || status.is_redirection() {
        debug!("List loaded on server.");
        Ok(())
    } else {
        error!("Couldn't load list on server because of status {status}...");
        Err(ApplicationError::from(CantLoadListOnServer))
    }
}

async fn retrieve_download_link(
    client: &Client,
    host: &str,
    download_link_regex: &Regex,
) -> Result<String> {
    let request = prepare_request_for_retrieving_download_link(client, host);
    let response = request.send().await.map_err(log_message_and_return(
        "Can't export list.",
        CantRetrieveDownloadLink,
    ))?;

    let status = response.status();
    if !status.is_success() && !status.is_redirection() {
        return Err(ApplicationError::from(CantRetrieveDownloadLink));
    }

    let page_content = response
        .text()
        .await
        .map_err(log_error_and_return(CantReadPageContent))?;
    let regex = download_link_regex;
    let file_url = regex.find(&page_content).ok_or(NoDownloadLink)?.as_str();
    Ok(file_url.to_owned())
}

async fn download_list(client: &Client, file_url: &str) -> Result<String> {
    let response = client
        .get(file_url)
        .send()
        .await
        .map_err(log_message_and_return("Can't download list.", NotFound))?;

    let status = response.status();
    if !status.is_success() && !status.is_redirection() {
        Err(NotFound)?;
    }

    let file_content_as_bytes = response
        .bytes()
        .await
        .map_err(log_error_and_return(MalformedMembershipsDownloadResponse))?;
    Ok(ISO_8859_1
        .decode(file_content_as_bytes.as_ref(), DecoderTrap::Strict)
        .map_err(FileoError::from)?)
}
// endregion

// region Requests preparation
fn prepare_request_for_connection(
    client: &Client,
    domain: &str,
    credentials: &FileoCredentials,
) -> RequestBuilder {
    let url = format!("{domain}/page.php");
    let arguments = [
        ("Action", "connect_user"),
        ("requestForm", "formConnecter"),
        ("login", credentials.login().as_str()),
        ("password", credentials.password().as_str()),
    ];
    let body = format_arguments_into_body(&arguments);
    client
        .post(&url)
        .header("Content-Type", ContentType::Form.to_string())
        .body(body)
}

fn prepare_request_for_loading_list_into_server_session(
    client: &Client,
    domain: &str,
) -> RequestBuilder {
    let url = format!("{domain}/page.php?P=bo/extranet/adhesion/annuaire/index");
    let arguments = [
        ("Action", "adherent_filtrer"),
        ("requestForm", "formFiltrer"),
        ("affich_select_nom", "3"),
        ("affich_text_nom", ""),
        ("affich_select_prenom", "3"),
        ("affich_text_prenom", ""),
        ("affich_select_majeur", ""),
        ("affich_text_numLicence", ""),
        ("affich_text_dateCreationFrom", ""),
        ("affich_text_dateCreationTo", ""),
        ("affich_text_dateDebut", ""),
        ("affich_text_dateFin", ""),
        ("affich_text_dateSaisieDebut", ""),
        ("affich_text_dateSaisieFin", ""),
        ("affich_radio_statut", ""),
        ("affich_select_regionStructure", ""),
        ("affich_select_departementStructure", ""),
        ("affich_select_code", "2"),
        ("affich_text_code", ""),
        ("affich_fixed_instanceId", "2012"),
        ("affich_radio_structFille", "1"),
        ("affich_select_typeAdhesion", ""),
        ("affich_select_tarif", ""),
        ("affich_select_regle", ""),
        ("affich_select_nomGroupe", "3"),
        ("affich_text_nomGroupe", ""),
    ];
    let body = format_arguments_into_body(&arguments);
    client
        .post(url)
        .header("Content-Type", ContentType::Form.to_string())
        .body(body)
}

fn prepare_request_for_retrieving_download_link(client: &Client, domain: &str) -> RequestBuilder {
    let url = format!("{domain}/includer.php?inc=ajax/adherent/adherent_export");
    let arguments = [
        ("requestForm", "formExport"),
        ("export_radio_format", "2"),
        ("option_checkbox_champs[nom]", "nom"),
        ("option_checkbox_champs[prenom]", "prenom"),
        ("option_checkbox_champs[sexe]", "sexe"),
        ("option_checkbox_champs[dateNaissance]", "dateNaissance"),
        ("option_checkbox_champs[age]", "age"),
        ("option_checkbox_champs[numeroLicence]", "numeroLicence"),
        ("option_checkbox_champs[email]", "email"),
        ("option_checkbox_champs[isAdhesionRegle]", "isAdhesionRegle"),
        ("option_checkbox_champs[dateAdhesionFin]", "dateAdhesionFin"),
        ("option_checkbox_champs[expire]", "expire"),
        ("option_checkbox_champs[instanceNom]", "instanceNom"),
        ("option_checkbox_champs[instanceCode]", "instanceCode"),
        ("generation", "2"),
    ];
    let body = format_arguments_into_body(&arguments);
    client
        .post(url)
        .header("Content-Type", ContentType::Form.to_string())
        .body(body)
}
// endregion

fn format_arguments_into_body(args: &[(&str, &str)]) -> String {
    args.iter()
        .map(|(key, value)| match value {
            &"" => key.to_string(),
            value => format!("{key}={value}"),
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn create_memberships_file_dir(memberships_file_folder: &OsStr) -> Result<()> {
    std::fs::create_dir_all(memberships_file_folder)
        .map_err(MembershipsFileFolderCreationFailed)?;

    Ok(())
}

fn write_list_to_file(members_file_folder: &OsStr, file_content: &str) -> Result<FileDetails> {
    let date_time = Local::now().date_naive();
    let filepath = PathBuf::from(members_file_folder)
        .join(format!("memberships-{}.csv", date_time.format("%Y-%m-%d")));
    std::fs::write(&filepath, file_content).map_err(MembershipsFileWriteFailed)?;
    Ok(FileDetails::new(date_time, OsString::from(filepath)))
}

fn parse_file(file_content: &str) -> Vec<Membership> {
    let reader = BufReader::new(file_content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(reader);
    load_memberships(&mut reader)
}

fn load_memberships<T>(reader: &mut Reader<T>) -> Vec<Membership>
where
    T: std::io::Read,
{
    reader
        .deserialize()
        .filter_map(|result: Result<ImportedMembership, _>| match result {
            Ok(membership) => Some(membership.into()),
            Err(e) => {
                log_message("Error while reading membership")(e);
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApplicationError;
    use crate::error::ApplicationError::{Fileo, Web};
    use crate::membership::config::MembershipsProviderConfig;
    use crate::membership::get_memberships_file_folder;
    use crate::tools::test::tests::temp_dir;
    use crate::web::error::WebError;
    use dto::membership::tests::{get_expected_membership, get_membership_as_csv};
    use encoding::EncoderTrap;
    use regex::Regex;
    use rocket::http::ContentType;
    use std::fs;
    use std::path::PathBuf;
    use std::time::SystemTime;
    use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[async_test]
    async fn should_download_members_list() {
        let mock_server = MockServer::start().await;

        let temp_dir = temp_dir();
        let config = MembershipsProviderConfig::new(
            mock_server.uri(),
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap(),
            temp_dir.into_os_string(),
        );
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let download_filename = "download.csv";
        let download_link = format!("{}/{download_filename}", mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "Profil Commission thématique - UNSLL - Commission Nationale Monocycle",
            ))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(query_param_contains(
                "P",
                "bo/extranet/adhesion/annuaire/index",
            ))
            .and(body_string_contains("Action=adherent_filtrer"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/includer.php"))
            .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                format!("<p>Here is the download link: {download_link}</p>"),
                "text/html",
            ))
            .mount(&mock_server)
            .await;
        let member_as_csv = get_membership_as_csv();
        let member_as_csv = ISO_8859_1
            .encode(&member_as_csv, EncoderTrap::Strict)
            .unwrap();
        let message_in_latin1: &[u8] = &member_as_csv;
        Mock::given(method("GET"))
            .and(path(format!("/{download_filename}").to_owned()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"))
            .mount(&mock_server)
            .await;

        let result = download_memberships_list(&config, &credentials)
            .await
            .unwrap();
        assert_eq!(vec![get_expected_membership()], result);
    }

    #[test]
    fn should_create_members_file_dir() {
        let path = temp_dir();
        let path = path.join(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        );
        fs::create_dir(&path).unwrap();
        let members_file_folder_path = path.join(get_memberships_file_folder());
        let result = create_memberships_file_dir(members_file_folder_path.as_ref());

        assert!(result.is_ok());
        assert!(fs::exists(members_file_folder_path).is_ok_and(|r| r));
    }

    #[test]
    fn should_build_client() {
        let result = build_client();
        assert!(result.is_ok());
    }

    // region Requests
    #[async_test]
    async fn should_connect() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "Profil Commission thématique - UNSLL - Commission Nationale Monocycle",
            ))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_ok());
    }

    #[async_test]
    async fn should_not_connect_when_internal_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_err_and(|e| matches!(e, Web(ConnectionFailed))));
    }

    #[async_test]
    async fn should_not_connect_when_credentials_dont_match() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("L'identifiant et le mot de passe ne correspondent pas"),
            )
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_err_and(|e| matches!(e, Web(WrongCredentials))));
    }

    #[async_test]
    async fn should_not_connect_when_missing_id() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("Le champ 'Identifiant' est obligatoire"),
            )
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_err_and(|e| matches!(e, Web(WrongCredentials))));
    }

    #[async_test]
    async fn should_not_connect_when_missing_password() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("Le champ 'Mot de passe' est obligatoire"),
            )
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_err_and(|e| matches!(e, Web(WrongCredentials))));
    }

    #[async_test]
    async fn should_not_connect_when_lack_of_permissions() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(body_string_contains("Action=connect_user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("Profil Club - The Best Unicycle Club Ever"),
            )
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();
        let credentials = FileoCredentials::new(String::new(), String::new());

        let result = login_to_fileo(&client, &mock_server.uri(), &credentials).await;
        assert!(result.is_err_and(|e| matches!(e, Web(WebError::LackOfPermissions))));
    }

    #[async_test]
    async fn should_load_list_into_server_session() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(query_param_contains(
                "P",
                "bo/extranet/adhesion/annuaire/index",
            ))
            .and(body_string_contains("Action=adherent_filtrer"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result = load_list_into_server_session(&client, &mock_server.uri()).await;
        assert!(result.is_ok());
    }

    #[async_test]
    async fn should_not_load_list_into_server_session() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/page.php"))
            .and(query_param_contains(
                "P",
                "bo/extranet/adhesion/annuaire/index",
            ))
            .and(body_string_contains("Action=adherent_filtrer"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result = load_list_into_server_session(&client, &mock_server.uri()).await;
        assert!(result.is_err_and(|e| matches!(e, Fileo(CantLoadListOnServer))));
    }

    #[async_test]
    async fn should_retrieve_download_link() {
        let mock_server = MockServer::start().await;
        let expected_link = format!("{}/download.csv", mock_server.uri());
        let download_link_regex =
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap();

        Mock::given(method("POST"))
            .and(path("/includer.php"))
            .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                format!("<p>Here is the download link: {expected_link}</p>"),
                "text/html",
            ))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result =
            retrieve_download_link(&client, &mock_server.uri(), &download_link_regex).await;
        assert!(result.is_ok_and(|link| link == expected_link));
    }

    #[async_test]
    async fn should_not_retrieve_download_link_when_internal_server_error() {
        let mock_server = MockServer::start().await;
        let download_link_regex =
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap();

        Mock::given(method("POST"))
            .and(path("/includer.php"))
            .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result =
            retrieve_download_link(&client, &mock_server.uri(), &download_link_regex).await;
        assert!(result.is_err_and(|e| matches!(e, Fileo(CantRetrieveDownloadLink))));
    }

    #[async_test]
    async fn should_not_retrieve_download_link_when_no_link_in_page() {
        let mock_server = MockServer::start().await;
        let download_link_regex =
            Regex::new(&format!("{}/download\\.csv", mock_server.uri())).unwrap();

        Mock::given(method("POST"))
            .and(path("/includer.php"))
            .and(query_param_contains("inc", "ajax/adherent/adherent_export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("Are ya lookin' for a link?".to_string(), "text/html"),
            )
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result =
            retrieve_download_link(&client, &mock_server.uri(), &download_link_regex).await;
        assert!(result.is_err_and(|e| matches!(e, Fileo(NoDownloadLink))));
    }

    #[async_test]
    async fn should_download_list() {
        let message_in_latin1: &[u8] = &[239]; // Represents the character `ï` in LATIN1/ISO_8859_1

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(message_in_latin1, "text/csv"))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result = download_list(&client, &mock_server.uri()).await;
        assert_eq!("ï", result.unwrap());
    }

    #[async_test]
    async fn should_not_download_list_when_file_not_found() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = build_client().unwrap();

        let result = download_list(&client, &mock_server.uri()).await;
        let error = result.err().unwrap();
        assert!(matches!(error, ApplicationError::Web(NotFound)));
    }

    #[test]
    fn should_write_list_to_file() {
        let temp_dir = temp_dir();
        let expected_content = "content;csv";

        let result = write_list_to_file(temp_dir.as_ref(), expected_content);

        let file_details = result.unwrap();
        let content = fs::read_to_string(file_details.filepath()).unwrap();
        assert_eq!(expected_content, content);
    }

    #[test]
    fn should_write_list_to_file_when_non_existing_folder() {
        let temp_dir = PathBuf::from("/this/path/does/not/exist");

        let result = write_list_to_file(temp_dir.as_ref(), "");

        assert!(matches!(
            result.err().unwrap(),
            Fileo(MembershipsFileWriteFailed(_))
        ));
    }
    // endregion

    // region Requests preparation
    #[test]
    fn should_prepare_request_for_connection() {
        let client = build_client().unwrap();
        let domain = "http://localhost:27001";
        let login = "login";
        let password = "password";
        let credentials = FileoCredentials::new(login.to_owned(), password.to_owned());

        let expected_body = format!(
            "Action=connect_user&requestForm=formConnecter&login={login}&password={password}"
        );

        let result = prepare_request_for_connection(&client, domain, &credentials);

        let result_request = result.build();
        assert!(result_request.is_ok());
        let request = result_request.unwrap();
        assert_eq!(
            expected_body,
            String::from_utf8_lossy(request.body().unwrap().as_bytes().unwrap())
        );
        assert_eq!(
            &ContentType::Form.to_string(),
            request
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn should_prepare_request_for_loading_list_into_server_session() {
        let client = build_client().unwrap();
        let domain = "http://localhost:27001";

        let expected_body = "Action=adherent_filtrer&requestForm=formFiltrer&affich_select_nom=3&affich_text_nom&affich_select_prenom=3&affich_text_prenom&affich_select_majeur&affich_text_numLicence&affich_text_dateCreationFrom&affich_text_dateCreationTo&affich_text_dateDebut&affich_text_dateFin&affich_text_dateSaisieDebut&affich_text_dateSaisieFin&affich_radio_statut&affich_select_regionStructure&affich_select_departementStructure&affich_select_code=2&affich_text_code&affich_fixed_instanceId=2012&affich_radio_structFille=1&affich_select_typeAdhesion&affich_select_tarif&affich_select_regle&affich_select_nomGroupe=3&affich_text_nomGroupe";

        let result = prepare_request_for_loading_list_into_server_session(&client, domain);

        let result_request = result.build();
        assert!(result_request.is_ok());
        let request = result_request.unwrap();
        assert_eq!(
            expected_body,
            String::from_utf8_lossy(request.body().unwrap().as_bytes().unwrap())
        );
        assert_eq!(
            &ContentType::Form.to_string(),
            request
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn should_prepare_request_for_retrieving_download_link() {
        let client = build_client().unwrap();
        let domain = "http://localhost:27001";

        let expected_body = "requestForm=formExport&export_radio_format=2&option_checkbox_champs[nom]=nom&option_checkbox_champs[prenom]=prenom&option_checkbox_champs[sexe]=sexe&option_checkbox_champs[dateNaissance]=dateNaissance&option_checkbox_champs[age]=age&option_checkbox_champs[numeroLicence]=numeroLicence&option_checkbox_champs[email]=email&option_checkbox_champs[isAdhesionRegle]=isAdhesionRegle&option_checkbox_champs[dateAdhesionFin]=dateAdhesionFin&option_checkbox_champs[expire]=expire&option_checkbox_champs[instanceNom]=instanceNom&option_checkbox_champs[instanceCode]=instanceCode&generation=2";

        let result = prepare_request_for_retrieving_download_link(&client, domain);

        let result_request = result.build();
        assert!(result_request.is_ok());
        let request = result_request.unwrap();
        assert_eq!(
            expected_body,
            String::from_utf8_lossy(request.body().unwrap().as_bytes().unwrap())
        );
        assert_eq!(
            &ContentType::Form.to_string(),
            request
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
    // endregion

    #[test]
    fn should_format_arguments_into_body() {
        let arguments = [("key1", "value1"), ("key2", "value2"), ("key3", "")];
        assert_eq!(
            "key1=value1&key2=value2&key3",
            format_arguments_into_body(&arguments)
        )
    }

    #[test]
    fn debug_credentials() {
        let credentials = FileoCredentials::new("login".to_owned(), "password".to_owned());
        assert_eq!(
            "Fileo Credentials {login=login, password=MASKED}",
            format!("{credentials:?}")
        );
    }

    mod load_memberships {
        use crate::fileo::download::load_memberships;
        use dto::membership::tests::{
            get_expected_membership, get_malformed_membership_as_csv, get_membership_as_csv,
        };
        use std::io::BufReader;

        #[test]
        fn success() {
            let entry = get_membership_as_csv();
            let expected_member = get_expected_membership();

            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b';')
                .from_reader(BufReader::new(entry.as_bytes()));
            let members = load_memberships(&mut reader);
            assert_eq!(vec![expected_member], members);
        }

        #[test]
        fn fail_when_malformed_input() {
            let entry = get_malformed_membership_as_csv();
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b';')
                .from_reader(BufReader::new(entry.as_bytes()));
            let members = load_memberships(&mut reader);
            assert!(members.is_empty(), "`members` is not empty.");
        }
    }
}
