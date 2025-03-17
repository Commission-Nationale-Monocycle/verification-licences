use crate::error::{ApplicationError, Result};
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::web::error::WebError::{ConnectionFailed, WrongCredentials};
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn authenticate_into_uda(
    client: &Client,
    base_url: &str,
    login: &str,
    password: &str,
) -> Result<()> {
    let authenticity_token = get_authenticity_token(client, base_url)
        .await
        .map_err(log_error_and_return(ConnectionFailed))?;

    check_credentials(client, base_url, &authenticity_token, login, password)
        .await
        .map_err(log_error_and_return(ApplicationError::from(
            WrongCredentials,
        )))
}

async fn get_authenticity_token(client: &Client, base_url: &str) -> Result<String> {
    let url = format!("{base_url}/en/users/sign_in");
    let response = client
        .get(url)
        .send()
        .await
        .map_err(log_message_and_return(
            "Can't get authenticity token from UDA",
            ConnectionFailed,
        ))?;

    let body = response
        .text()
        .await
        .map_err(log_error_and_return(ConnectionFailed))?;

    let document = Html::parse_document(&body);
    let authenticity_token = get_authenticity_token_from_html(&document).map_err(
        log_message_and_return("Can't get authenticity token from UDA", ConnectionFailed),
    )?;

    Ok(authenticity_token.to_owned())
}

fn get_authenticity_token_from_html(document: &Html) -> Result<&str> {
    let token_selector = Selector::parse(r#"input[name="authenticity_token"]"#).unwrap();
    let element = document.select(&token_selector).next().ok_or_else(|| {
        error!("Authenticity token not found");
        ConnectionFailed
    })?;
    let authenticity_token = element.value().attr("value").unwrap();
    Ok(authenticity_token)
}

async fn check_credentials(
    client: &Client,
    base_url: &str,
    authenticity_token: &str,
    login: &str,
    password: &str,
) -> Result<()> {
    let url = format!("{}/en/users/sign_in", base_url);
    let params = [
        ("user[email]", login),
        ("user[password]", password),
        ("authenticity_token", authenticity_token),
        ("utf8", "✓"),
    ];
    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(log_message_and_return(
            "Failed to authenticate to UDA [user: {login}]",
            ConnectionFailed,
        ))?;

    let status = response.status();
    if status.is_success() {
        let text = response.text().await.map_err(log_message_and_return(
            "Failed to authenticate to UDA",
            ConnectionFailed,
        ))?;
        if text.contains("Signed in successfully") || text.contains("You are already signed in") {
            debug!("Logged in UDA [user: {login}]");
            Ok(())
        } else if text.contains("Invalid User Account Email or password") {
            error!("Failed to authenticate to UDA. Wrong credentials? [user: {login}]");
            Err(ApplicationError::from(WrongCredentials))
        } else {
            error!(
                "Failed to authenticate to UDA. Unknown error. See response body: {}",
                text
            );
            Err(ApplicationError::from(ConnectionFailed))
        }
    } else {
        error!("Failed to authenticate to UDA. Is the instance up? [user: {login}]");
        Err(ApplicationError::from(ConnectionFailed))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::error::ApplicationError::Web;
    use crate::tools::web::build_client;
    use crate::web::credentials::UdaCredentials;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // region Common methods and const
    const AUTHENTICITY_TOKEN: &str =
        "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

    pub async fn setup_authentication(mock_server: &MockServer) -> UdaCredentials {
        let authenticity_token = setup_authenticity_token(mock_server).await;
        setup_check_credentials(mock_server, &authenticity_token).await
    }

    async fn setup_check_credentials(
        mock_server: &MockServer,
        authenticity_token: &str,
    ) -> UdaCredentials {
        let login = "login";
        let password = "password";

        let params = format!(
            "user%5Bemail%5D={login}&user%5Bpassword%5D={password}&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string("Signed in successfully"))
            .mount(mock_server)
            .await;

        UdaCredentials::new(mock_server.uri(), login.to_owned(), password.to_owned())
    }

    pub(crate) async fn setup_authenticity_token(mock_server: &MockServer) -> String {
        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{AUTHENTICITY_TOKEN}"></body></html>"#
        );
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(mock_server)
            .await;

        AUTHENTICITY_TOKEN.to_owned()
    }
    // endregion

    // region authenticate_into_uda
    #[async_test]
    async fn should_authenticate_into_uda() {
        let mock_server = MockServer::start().await;
        let credentials = setup_authentication(&mock_server).await;

        let client = Client::new();
        authenticate_into_uda(
            &client,
            credentials.uda_url(),
            credentials.login(),
            credentials.password(),
        )
        .await
        .unwrap();
    }

    #[async_test]
    async fn should_fail_to_authenticate_into_uda_when_connection_failed() {
        let login = "login";
        let password = "password";

        let mock_server = MockServer::start().await;
        let credentials =
            UdaCredentials::new(mock_server.uri(), login.to_owned(), password.to_owned());

        let client = Client::new();
        let error = authenticate_into_uda(
            &client,
            credentials.uda_url(),
            credentials.login(),
            credentials.password(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, Web(ConnectionFailed)));
    }

    #[async_test]
    async fn should_fail_to_authenticate_into_uda_when_wrong_credentials() {
        let login = "login";
        let password = "password";

        let mock_server = MockServer::start().await;
        let authenticity_token = setup_authenticity_token(&mock_server).await;
        let params = format!(
            "user%5Bemail%5D={login}&user%5Bpassword%5D={password}&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<html><body>Invalid User Account Email or password</body></html>",
            ))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let error = authenticate_into_uda(&client, &mock_server.uri(), login, password)
            .await
            .unwrap_err();

        assert!(matches!(error, Web(WrongCredentials)));
    }
    // endregion

    // region get_authenticity_token
    #[async_test]
    async fn should_get_authenticity_token() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        let expected_token = setup_authenticity_token(&mock_server).await;

        let token = get_authenticity_token(&client, &mock_server.uri())
            .await
            .unwrap();
        assert_eq!(expected_token, token);
    }

    #[async_test]
    async fn should_not_get_authenticity_token_when_unreachable() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();

        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = get_authenticity_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert!(matches!(error, Web(ConnectionFailed)));
    }

    #[async_test]
    async fn should_not_get_authenticity_token_not_in_page() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();

        let body = "<html><body><div>What are ya lookin' for, son?</div></body></html>";
        Mock::given(method("GET"))
            .and(path("/en/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = get_authenticity_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert!(matches!(error, Web(ConnectionFailed)));
    }
    // endregion

    // region get_authenticity_token_from_html
    #[test]
    fn should_get_authenticity_token_from_html() {
        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{AUTHENTICITY_TOKEN}"></body></html>"#
        );
        let html = Html::parse_document(&body);
        let token = get_authenticity_token_from_html(&html).unwrap();

        assert_eq!(AUTHENTICITY_TOKEN, token);
    }

    #[test]
    fn should_not_get_authenticity_token_from_html() {
        let body = "<html><body><div>What are ya lookin' for, son?</div></body></html>";
        let html = Html::parse_document(body);
        let error = get_authenticity_token_from_html(&html).unwrap_err();

        assert!(matches!(error, Web(ConnectionFailed)));
    }
    // endregion

    // region check_credentials
    #[async_test]
    async fn should_check_credentials() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;

        setup_check_credentials(&mock_server, AUTHENTICITY_TOKEN).await;

        check_credentials(
            &client,
            &mock_server.uri(),
            AUTHENTICITY_TOKEN,
            "login",
            "password",
        )
        .await
        .unwrap();
    }

    #[async_test]
    async fn should_fail_to_check_credentials_when_wrong_credentials() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={AUTHENTICITY_TOKEN}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<html><body>Invalid User Account Email or password</body></html>",
            ))
            .mount(&mock_server)
            .await;

        let error = check_credentials(
            &client,
            &mock_server.uri(),
            AUTHENTICITY_TOKEN,
            "login",
            "password",
        )
        .await
        .unwrap_err();
        assert!(matches!(error, Web(WrongCredentials)));
    }

    #[async_test]
    async fn should_fail_to_check_credentials_when_other_error() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(path("/en/users/sign_in"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = check_credentials(
            &client,
            &mock_server.uri(),
            authenticity_token,
            "login",
            "password",
        )
        .await
        .unwrap_err();
        assert!(matches!(error, Web(ConnectionFailed)));
    }
    // endregion
}
