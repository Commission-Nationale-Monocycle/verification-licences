use std::env;
use log::{debug, error, warn};
use reqwest::blocking::Client;

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
                login = Some(arg[arg.find("=").unwrap()..].to_string());
            }
            if arg.starts_with("--password=") || arg.starts_with("-p=") {
                password = Some(arg[arg.find("=").unwrap()..].to_string());
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

pub fn connect() -> Result<(), ()> {
    let credentials = match retrieve_credentials() {
        None => { return Err(()); }
        Some(credentials) => { credentials }
    };

    let connection_url = format!("{URL_DOMAIN}/page.php");
    let client = build_client();
    match client.post(connection_url).query(
        &[
            ("Action", "connect_user"),
            ("requestForm", "formConnecter"),
            ("login", credentials.login.as_str()),
            ("password", credentials.password.as_str()),
            ("P", "bo%2Fextranet%2Fstructure%2Fannuaire%2Findex")
        ]
    )
        .send() {
        Ok(_) => {
            debug!("Connected!");
            return Ok(());
        }
        Err(e) => {
            error!("Connection failed...");
            error!("{}", e.to_string());
            panic!("Connection failed, aborting process.")
        }
    }
}