use crate::demo_mock_server::fileo::init_fileo_mock_server;
use crate::demo_mock_server::uda::init_uda_mock_server;
use std::sync::OnceLock;

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

mod fileo {
    use crate::demo_mock_server::{DEMO_FILE, FILEO_MOCK_SERVER_URI};
    use encoding::all::ISO_8859_1;
    use encoding::{EncoderTrap, Encoding};
    use wiremock::matchers::{body_string_contains, method, path, query_param_contains};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    pub async fn init_fileo_mock_server() -> MockServer {
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
}

mod uda {
    use crate::demo_mock_server::UDA_MOCK_SERVER_URI;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const AUTHENTICITY_TOKEN: &str =
        "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

    pub async fn init_uda_mock_server() -> MockServer {
        let mock_server = MockServer::start().await;
        UDA_MOCK_SERVER_URI.get_or_init(|| mock_server.uri());

        mock_uda_login(&mock_server).await;
        mock_uda_retrieve_members(&mock_server).await;

        mock_server
    }

    async fn mock_uda_login(mock_server: &MockServer) {
        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{AUTHENTICITY_TOKEN}"></body></html>"#
        );
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Signed in successfully"))
            .mount(&mock_server)
            .await;
    }

    async fn mock_uda_retrieve_members(mock_server: &MockServer) {
        let expected_id_1 = "123456";
        let expected_first_name_1 = "Jon";
        let expected_name_1 = "DOE";
        let expected_id_2 = "654321";
        let expected_first_name_2 = "Jonette";
        let expected_name_2 = "Snow";

        let body = format!(
            r##"<html><body><h1>Unicycling Society/Federation Membership Management</h1><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id_1}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name_1}</td><td>{expected_name_1}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr><tr class="even" id="reg_1" role="row"><td><a href="/fr/registrants/1">1</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_1" id="membership_number_1">ID #{expected_id_2}</span><span class="is--hidden" id="member_number_form_1"><form action="/fr/organization_memberships/1/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name_2}</td><td>{expected_name_2}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/1/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );

        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;
    }
}
