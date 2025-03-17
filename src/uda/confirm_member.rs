use crate::error::{ApplicationError, Result};
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::uda::error::UdaError;
use crate::uda::error::UdaError::{MemberConfirmationFailed, OrganizationMembershipsAccessFailed};
use crate::web::error::WebError::{CantReadPageContent, ConnectionFailed, LackOfPermissions};
use reqwest::{Client, StatusCode};
use rocket::form::validate::Contains;
use scraper::{Html, Selector};

/// Try and mark member as confirmed on UDA.
/// If called on a member already confirmed, it marks them as unconfirmed before trying to mark it as confirmed again.
pub async fn confirm_member(client: &Client, base_url: &str, id: u16) -> Result<()> {
    let csrf_token = get_csrf_token(client, base_url).await?;
    confirm_member_with_retry(client, base_url, id, &csrf_token, true).await
}

async fn confirm_member_with_retry(
    client: &Client,
    base_url: &str,
    id: u16,
    csrf_token: &str,
    should_retry: bool,
) -> Result<()> {
    let url = format!("{base_url}/en/organization_memberships/{id}/toggle_confirm");
    let response = client
        .put(url)
        .header("Accept", "*/*;q=0.5, text/javascript, application/javascript, application/ecmascript, application/x-ecmascript")
        .header("X-CSRF-Token", csrf_token)
        .send()
        .await
        .map_err(log_message_and_return(
            "Can't mark as confirmed on UDA",
            ConnectionFailed,
        ))?;

    let status = response.status();
    if !status.is_success() {
        warn!("Can't mark as confirmed on UDA [status: {status}]");
        return match status {
            StatusCode::NOT_FOUND => Err(ApplicationError::from(LackOfPermissions)), // If the user is not authorized to confirm members, then we get a 404...
            _ => Err(ApplicationError::from(ConnectionFailed)),
        };
    }

    let body = response.text().await.map_err(log_message_and_return(
        "Can't read text after having marked user as confirmed",
        CantReadPageContent,
    ))?;

    let unmarked_message = format!(
        r#"href=\"/en/organization_memberships/{id}/toggle_confirm\">Mark as confirmed<\/a>"#
    );
    let marked_message = format!(
        r#"href=\"/en/organization_memberships/{id}/toggle_confirm\">Mark as unconfirmed<\/a>"#
    );
    if body.contains(unmarked_message.as_str()) {
        if should_retry {
            warn!(
                "Member has been unconfirmed! Trying to confirm them back. [uda_url: {base_url}, id: {id}]"
            );
            Box::pin(confirm_member_with_retry(
                client, base_url, id, csrf_token, false,
            ))
            .await
        } else {
            error!(
                "Member has been unconfirmed! NOT trying to confirm them back. [uda_url: {base_url}, id: {id}]"
            );
            Err(MemberConfirmationFailed(id))?
        }
    } else if body.contains(marked_message.as_str()) {
        trace!("Member has been confirmed on UDA! [uda_url: {base_url}, id: {id}]");
        Ok(())
    } else {
        Err(ApplicationError::from(LackOfPermissions))
    }
}

async fn get_csrf_token(client: &Client, base_url: &str) -> Result<String> {
    let url = format!("{base_url}/en/organization_memberships");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(log_error_and_return(OrganizationMembershipsAccessFailed))?;

    let status = response.status();
    if status.is_success() {
        let body = response.text().await.map_err(log_message_and_return(
            "Can't read organization_memberships content",
            OrganizationMembershipsAccessFailed,
        ))?;
        if body.contains("Unicycling Society/Federation Membership Management") {
            retrieve_csrf_from_html(&body).await
        } else {
            error!("Can't access organization_memberships page. Lack of permissions?");
            Err(ApplicationError::from(LackOfPermissions))
        }
    } else {
        error!(
            "Can't reach organization_memberships page: {:?}",
            response.status()
        );
        Err(OrganizationMembershipsAccessFailed)?
    }
}

async fn retrieve_csrf_from_html(body: &str) -> Result<String> {
    let selector = Selector::parse(r#"meta[name="csrf-token"]"#).map_err(UdaError::from)?;
    let document = Html::parse_document(body);

    document
        .select(&selector)
        .next()
        .ok_or_else(|| {
            error!("Can't select CSRF token");
            ApplicationError::from(LackOfPermissions)
        })?
        .attr("content")
        .map(str::to_owned)
        .ok_or_else(|| {
            error!("Can't select CSRF token");
            ApplicationError::from(LackOfPermissions)
        })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::error::ApplicationError::{Uda, Web};
    use crate::tools::web::build_client;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    pub async fn setup_csrf_token(mock_server: &MockServer) -> String {
        let csrf_token = "PDKOFSqmdfjsdf3435dqs";
        let body = format!(
            r#"<html><head><meta name="csrf-token" content="{csrf_token}"></head><body>Unicycling Society/Federation Membership Management</body></html>"#
        );

        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(mock_server)
            .await;

        csrf_token.to_owned()
    }

    pub async fn setup_confirm_member(mock_server: &MockServer, csrf_token: &str, id: u16) {
        let body = format!(
            r##"var new_row = $("<tr class=\'confirmed\' id=\'reg_{id}\'>\n<td><a href=\"/en/registrants/{id}\">{id}<\/a><\/td>\n<td>\n<span class=\'member_number js--toggle\' data-toggle-target=\'#member_number_form_{id}\' id=\'membership_number_{id}\'>ID #012048<\/span>\n<span class=\'is--hidden\' id=\'member_number_form_1\'>\n<form action=\"/en/organization_memberships/1/update_number\" accept-charset=\"UTF-8\" data-remote=\"true\" method=\"post\"><input name=\"utf8\" type=\"hidden\" value=\"&#x2713;\" autocomplete=\"off\" /><input type=\"hidden\" name=\"_method\" value=\"put\" autocomplete=\"off\" /><input type=\"hidden\" name=\"authenticity_token\" value=\"fCnx1Z3o3n1jCeFbXxvniRDDcGt5wdQPNad8KQalzWw0qE3N56Q39nfPpoBG5fPXtu6RaSrDdUAvIkgOzCa5ug\" autocomplete=\"off\" /><input type=\"text\" name=\"membership_number\" id=\"membership_number\" value=\"012048\" />\n<input type=\"submit\" name=\"commit\" value=\"Update Membership #\" class=\"button tiny\" data-disable-with=\"Update Membership #\" />\n<\/form><\/span>\n<\/td>\n<td>François<\/td>\n<td>WURMSER<\/td>\n<td>34<\/td>\n<td>1985-03-20<\/td>\n<td>LA RICHE<\/td>\n<td>Indre-et-Loire<\/td>\n<td>France<\/td>\n<td>Roule Ta Bille<\/td>\n<td>\ntrue\n<\/td>\n<td>\nManually Confirmed\n<br>\n<a data-remote=\"true\" rel=\"nofollow\" data-method=\"put\" href=\"/en/organization_memberships/{id}/toggle_confirm\">Mark as unconfirmed<\/a>\n<\/td>\n<\/tr>\n")
old_row = $("#reg_{id}")
old_row.replaceWith(new_row)
new_row.effect("highlight", {{}}, 3000);
console.log("Updated {id}");"##
        );

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), csrf_token))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(mock_server)
            .await;
    }

    // region confirm_member
    #[async_test]
    async fn should_confirm_member() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        let csrf_token = setup_csrf_token(&mock_server).await;
        setup_confirm_member(&mock_server, &csrf_token, id).await;

        confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap();
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_lack_of_permissions() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();
        let csrf_token = setup_csrf_token(&mock_server).await;

        let body = "<div id='main'>
<p class='alert_flash'>You are not authorized to perform this action.</p>

I'm sorry, but the page you are searching for was not found.

</div>";

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), csrf_token))
            .respond_with(ResponseTemplate::new(404).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert!(matches!(error, Web(LackOfPermissions)));
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_connection_failed() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        let csrf_token = setup_csrf_token(&mock_server).await;

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), csrf_token))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert!(matches!(error, Web(ConnectionFailed)));
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_no_body() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();
        let csrf_token = setup_csrf_token(&mock_server).await;

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), csrf_token))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert!(matches!(error, Web(LackOfPermissions)));
    }

    #[async_test]
    async fn should_fail_to_confirm_member_and_not_retry_twice() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();
        let csrf_token = setup_csrf_token(&mock_server).await;

        let body = format!(
            r##"var new_row = $("<tr class=\'confirmed\' id=\'reg_1\'>\n<td><a href=\"/en/registrants/1\">1<\/a><\/td>\n<td>\n<span class=\'member_number js--toggle\' data-toggle-target=\'#member_number_form_1\' id=\'membership_number_1\'>ID #012048<\/span>\n<span class=\'is--hidden\' id=\'member_number_form_1\'>\n<form action=\"/en/organization_memberships/1/update_number\" accept-charset=\"UTF-8\" data-remote=\"true\" method=\"post\"><input name=\"utf8\" type=\"hidden\" value=\"&#x2713;\" autocomplete=\"off\" /><input type=\"hidden\" name=\"_method\" value=\"put\" autocomplete=\"off\" /><input type=\"hidden\" name=\"authenticity_token\" value=\"fCnx1Z3o3n1jCeFbXxvniRDDcGt5wdQPNad8KQalzWw0qE3N56Q39nfPpoBG5fPXtu6RaSrDdUAvIkgOzCa5ug\" autocomplete=\"off\" /><input type=\"text\" name=\"membership_number\" id=\"membership_number\" value=\"012048\" />\n<input type=\"submit\" name=\"commit\" value=\"Update Membership #\" class=\"button tiny\" data-disable-with=\"Update Membership #\" />\n<\/form><\/span>\n<\/td>\n<td>François<\/td>\n<td>WURMSER<\/td>\n<td>34<\/td>\n<td>1985-03-20<\/td>\n<td>LA RICHE<\/td>\n<td>Indre-et-Loire<\/td>\n<td>France<\/td>\n<td>Roule Ta Bille<\/td>\n<td>\ntrue\n<\/td>\n<td>\nManually Confirmed\n<br>\n<a data-remote=\"true\" rel=\"nofollow\" data-method=\"put\" href=\"/en/organization_memberships/{id}/toggle_confirm\">Mark as confirmed<\/a>\n<\/td>\n<\/tr>\n")
old_row = $("#reg_1")
old_row.replaceWith(new_row)
new_row.effect("highlight", {{}}, 3000);
console.log("Updated 1"");"##
        );

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), csrf_token))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        match error {
            Uda(MemberConfirmationFailed(error_id)) => assert_eq!(id, error_id),
            _ => panic!("Unexpected error"),
        }
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_wrong_csrf_token() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        setup_csrf_token(&mock_server).await;

        let body = "<div id='main'>
<p class='alert_flash'>You are not authorized to perform this action.</p>

I'm sorry, but the page you are searching for was not found.

</div>";

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .and(header("X-CSRF-Token".to_owned(), "Oops"))
            .respond_with(ResponseTemplate::new(401).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();

        assert!(matches!(error, Web(LackOfPermissions)));
    }
    // endregion

    // region get_csrf_token
    #[async_test]
    async fn should_get_csrf_token() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        let expected_csrf_token = setup_csrf_token(&mock_server).await;

        let result = get_csrf_token(&client, &mock_server.uri()).await.unwrap();
        assert_eq!(expected_csrf_token, result);
    }

    #[async_test]
    async fn should_fail_to_get_csrf_token_when_unreachable() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = get_csrf_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert!(matches!(error, Uda(OrganizationMembershipsAccessFailed)));
    }

    #[async_test]
    async fn should_fail_to_get_csrf_token_when_lack_of_permissions() {
        let body = "<html><body>You should log in to access this page.</body></html>";

        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = get_csrf_token(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert!(matches!(error, Web(LackOfPermissions)));
    }
    // endregion

    //region retrieve_csrf_from_html
    #[async_test]
    async fn should_retrieve_csrf_from_html() {
        let expected_csrf_token = "PDKOFSqmdfjsdf3435dqs";
        let html = format!(
            r#"<html><head><meta name="csrf-token" content="{expected_csrf_token}"></head><body></body></html>"#
        );

        let result = retrieve_csrf_from_html(&html).await.unwrap();
        assert_eq!(expected_csrf_token, result);
    }

    #[async_test]
    async fn should_fail_to_retrieve_csrf_from_html_when_no_tag() {
        let html = "<html><head></head><body></body></html>";

        let error = retrieve_csrf_from_html(html).await.unwrap_err();
        assert!(matches!(error, Web(LackOfPermissions)));
    }

    #[async_test]
    async fn should_fail_to_retrieve_csrf_from_html_when_empty_tag() {
        let html = r#"<html><head><meta name="csrf-token"></head><body></body></html>"#;

        let error = retrieve_csrf_from_html(html).await.unwrap_err();
        assert!(matches!(error, Web(LackOfPermissions)));
    }
    // endregion
}
