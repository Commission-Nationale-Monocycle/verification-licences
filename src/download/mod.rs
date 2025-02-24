use std::env;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use chrono::Local;
use log::{debug, error, info, warn};
use reqwest::blocking::Client;
use regex::Regex;

const URL_DOMAIN: &str = "https://www.leolagrange-fileo.org";

struct Credentials {
    login: String,
    password: String,
}

impl Credentials {
    pub fn new(login: String, password: String) -> Self {
        Self { login, password }
    }
}

fn build_client() -> Client {
    reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .unwrap()
}

fn retrieve_credentials() -> Option<Credentials> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() < 3 {
        warn!("Args don't contain login or password. It won't be possible to retrieve the members list.");
        None
    } else {
        let mut login = None;
        let mut password = None;
        for arg in &args {
            if arg.starts_with("--login=") || arg.starts_with("-l=") {
                login = Some(arg[arg.find("=").unwrap() + 1..].to_string());
            }
            if arg.starts_with("--password=") || arg.starts_with("-p=") {
                password = Some(arg[arg.find("=").unwrap() + 1..].to_string());
            }
        }

        if login.is_some() && password.is_some() {
            Some(Credentials::new(login.unwrap(), password.unwrap()))
        } else {
            warn!("Args don't contain login or password. It won't be possible to retrieve the members list.");
            None
        }
    }
}

pub fn connect() -> Result<Client, ()> {
    let credentials = match retrieve_credentials() {
        None => { return Err(()); }
        Some(credentials) => { credentials }
    };

    let url = format!("{URL_DOMAIN}/page.php");
    let client = build_client();
    let arguments = [
        ("Action", "connect_user"),
        ("requestForm", "formConnecter"),
        ("login", credentials.login.as_str()),
        ("password", credentials.password.as_str())
    ];
    let body = format_arguments_into_body(&arguments);
    let query = client.post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body);
    match query
        .send() {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                debug!("Connected!");
                return Ok(client);
            } else {
                error!("Connection failed...");
                panic!("Connection failed, aborting process.")
            }
        }
        Err(e) => {
            error!("Connection failed...");
            error!("{}", e.to_string());
            panic!("Connection failed, aborting process.")
        }
    }
}

pub fn get_list(client: &Client) -> Result<(), ()> {
    let url = format!("{URL_DOMAIN}/page.php?P=bo/extranet/adhesion/annuaire/index");
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
    match client.post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send() {
        Ok(_) => {
            debug!("Retrieved list!");
        }
        Err(e) => {
            error!("Can't retrieve list");
            error!("{}", e.to_string());
            panic!("Aborting process.")
        }
    };

    Ok(())
}

pub fn export_list(client: &Client) -> Result<String, ()> {
    let url = format!("{URL_DOMAIN}/includer.php?inc=ajax/adherent/adherent_export");
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
    let response = match client.post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send() {
        Ok(response) => {
            debug!("Export is ready!");
            response
        }
        Err(e) => {
            error!("Can't retrieve list");
            error!("{}", e.to_string());
            panic!("Aborting process.")
        }
    };

    let page_content = response.text().unwrap();
    let regex = Regex::new("https://www.leolagrange-fileo.org/clients/fll/telechargements/temp/.*?\\.csv").unwrap();
    let file_url = regex.find(&page_content).unwrap().as_str();
    match client.get(file_url).send() {
        Ok(response) => {
            let filename = format!("members-{}.csv", Local::now().format("%Y-%m%d"));
            dbg!(&filename);
            let mut file = File::create(&filename).unwrap();
            file.write(response.bytes().unwrap().as_ref());
            return Ok(filename);
        }
        Err(error) => {
            dbg!(&error);
            Err(())
        }
    }
}

fn format_arguments_into_body(args: &[(&str, &str)]) -> String {
    args.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join("&")
}

#[cfg(test)]
mod tests {
    use crate::download::format_arguments_into_body;

    #[test]
    fn should_format_arguments_into_body() {
        let arguments = [("key1", "value1"), ("key2", "value2")];
        assert_eq!("key1=value1&key2=value2", format_arguments_into_body(&arguments))
    }
}