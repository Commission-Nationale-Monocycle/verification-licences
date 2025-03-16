use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};
use std::sync::OnceLock;
use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub static FILEO_MOCK_SERVER_URI: OnceLock<String> = OnceLock::new();
pub static UDA_MOCK_SERVER_URI: OnceLock<String> = OnceLock::new();

const DEMO_FILE: &str = "Nom d'usage;Prénom;Sexe;Date de Naissance;Age;Numéro d'adhérent;Email;Réglé;Date Fin d'adhésion;Adherent expiré;Nom de structure;Code de structure
Doe;Jon;H;01-02-1980;45;123456;jon@doe.com;Oui;30-09-2025;Non;My club;Z01234
Bob;Alice;F;01-02-2000;25;987654;alice@bobo.com;Non;25-08-2024;Non;Her club;A98765";

pub async fn init_demo() {
    let mut fileo_server = init_fileo_mock_server().await;
    let mut uda_server = init_uda_mock_server().await;

    // This is a simple precaution: in some edge cases, both uri could be the same.
    // In such a case, the mocking would not work.
    while fileo_server.uri() == uda_server.uri() {
        fileo_server = init_fileo_mock_server().await;
        uda_server = init_uda_mock_server().await;
    }
}

// region Fileo
async fn init_fileo_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;
    FILEO_MOCK_SERVER_URI.get_or_init(|| mock_server.uri());
    mock_fileo_login(&mock_server).await;
    mock_download_members_list(&mock_server).await;

    mock_server
}

async fn mock_fileo_login(mock_server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/page.php"))
        .and(body_string_contains("Action=connect_user"))
        .respond_with(ResponseTemplate::new(200))
        .mount(mock_server)
        .await;
}

async fn mock_download_members_list(mock_server: &MockServer) {
    let download_filename = "download.csv";
    let download_link = format!("{}/{download_filename}", mock_server.uri());

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
    let message_in_latin1 = ISO_8859_1.encode(DEMO_FILE, EncoderTrap::Strict).unwrap();
    Mock::given(method("GET"))
        .and(path(format!("/{download_filename}").to_owned()))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(message_in_latin1.as_slice(), "text/csv"),
        )
        .mount(&mock_server)
        .await;
}
// endregion

// region UDA
async fn init_uda_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;
    UDA_MOCK_SERVER_URI.get_or_init(|| mock_server.uri());

    mock_server
}
// endregion
