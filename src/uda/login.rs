use crate::tools::error::Error::{ConnectionFailed, WrongCredentials};
use crate::tools::error::Result;
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::web::credentials::Credentials;
use reqwest::Client;
use scraper::{Html, Selector};

#[allow(dead_code)]
async fn get_authenticity_token(client: &Client, base_url: &str) -> Result<String> {
    let url = format!("{base_url}/users/sign_in");
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

#[allow(dead_code)]
fn get_authenticity_token_from_html(document: &Html) -> Result<&str> {
    let token_selector = Selector::parse(r#"input[name="authenticity_token"]"#).unwrap();
    let element = document.select(&token_selector).next().ok_or_else(|| {
        error!("Authenticity token not found");
        ConnectionFailed
    })?;
    let authenticity_token = element.value().attr("value").unwrap();
    Ok(authenticity_token)
}

#[allow(dead_code)]
async fn login(
    client: &Client,
    base_url: &str,
    authenticity_token: &str,
    credentials: &Credentials,
) -> Result<()> {
    let url = format!("{}/users/sign_in", base_url);
    let params = [
        ("user[email]", credentials.login().as_str()),
        ("user[password]", credentials.password().as_str()),
        ("authenticity_token", authenticity_token),
        ("utf8", "✓"),
    ];
    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(log_message_and_return(
            "Failed to authenticate to UDA",
            ConnectionFailed,
        ))?;

    let status = response.status();
    if status.is_success() {
        let text = response.text().await.map_err(log_message_and_return(
            "Failed to authenticate to UDA",
            ConnectionFailed,
        ))?;
        if text.contains("Signed in successfully") || text.contains("You are already signed in") {
            debug!("Logged in UDA.");
            Ok(())
        } else {
            error!("Failed to authenticate to UDA. Wrong credentials?");
            Err(WrongCredentials)
        }
    } else {
        error!("Failed to authenticate to UDA. Is the instance up?");
        Err(ConnectionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::web::build_client;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // region get_authenticity_token
    #[async_test]
    async fn should_get_authenticity_token() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        let expected_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{expected_token}"></body></html>"#
        );
        Mock::given(method("GET"))
            .and(path("/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .mount(&mock_server)
            .await;

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
            .and(path("/users/sign_in"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = get_authenticity_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert_eq!(ConnectionFailed, error);
    }

    #[async_test]
    async fn should_not_get_authenticity_token_not_in_page() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();

        let body = "<html><body><div>What are ya lookin' for, son?</div></body></html>";
        Mock::given(method("GET"))
            .and(path("/users/sign_in"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = get_authenticity_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert_eq!(ConnectionFailed, error);
    }
    // endregion

    // region get_authenticity_token_from_html
    #[test]
    fn should_get_authenticity_token_from_html() {
        let expected_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";

        let body = format!(
            r#"<html><body><input name="authenticity_token" value="{expected_token}"></body></html>"#
        );
        let html = Html::parse_document(&body);
        let token = get_authenticity_token_from_html(&html).unwrap();

        assert_eq!(expected_token, token);
    }

    #[test]
    fn should_not_get_authenticity_token_from_html() {
        let body = "<html><body><div>What are ya lookin' for, son?</div></body></html>";
        let html = Html::parse_document(body);
        let error = get_authenticity_token_from_html(&html).unwrap_err();

        assert_eq!(ConnectionFailed, error);
    }
    // endregion

    // region login
    #[async_test]
    async fn should_login() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";
        let credentials = Credentials::new("login".to_owned(), "password".to_owned());

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200).set_body_string("Signed in successfully"))
            .mount(&mock_server)
            .await;

        let result = login(
            &client,
            &mock_server.uri(),
            authenticity_token,
            &credentials,
        )
        .await;
        assert_eq!(Ok(()), result);
    }

    #[async_test]
    async fn should_fail_to_login_when_wrong_credentials() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";
        let credentials = Credentials::new("login".to_owned(), "password".to_owned());

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let result = login(
            &client,
            &mock_server.uri(),
            authenticity_token,
            &credentials,
        )
        .await;
        assert_eq!(WrongCredentials, result.unwrap_err());
    }

    #[async_test]
    async fn should_fail_to_login_when_other_error() {
        let client = build_client().unwrap();
        let mock_server = MockServer::start().await;
        let authenticity_token = "BDv-07yMs8kMDnRn2hVgpSmqn88V_XhCZxImtcXr3u6OOmpnsy0WpFD49rTOuOEfJG_PptBBJag094Vd0uuyZg";
        let credentials = Credentials::new("login".to_owned(), "password".to_owned());

        let params = format!(
            "user%5Bemail%5D=login&user%5Bpassword%5D=password&authenticity_token={authenticity_token}&utf8=%E2%9C%93"
        );
        Mock::given(method("POST"))
            .and(body_string(&params))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = login(
            &client,
            &mock_server.uri(),
            authenticity_token,
            &credentials,
        )
        .await;
        assert_eq!(ConnectionFailed, result.unwrap_err());
    }
    // endregion
}
