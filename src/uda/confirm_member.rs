use crate::tools::error::Error::{
    CantReadPageContent, CartMarkMember, ConnectionFailed, LackOfPermissions,
};
use crate::tools::error::Result;
use crate::tools::log_message_and_return;
use reqwest::{Client, StatusCode};
use rocket::form::validate::Contains;

#[allow(dead_code)]
pub async fn confirm_member(client: &Client, base_url: &str, id: u16) -> Result<()> {
    confirm_member_with_retry(client, base_url, id, true).await
}

async fn confirm_member_with_retry(
    client: &Client,
    base_url: &str,
    id: u16,
    should_retry: bool,
) -> Result<()> {
    let url = format!("{base_url}/en/organization_memberships/{id}/toggle_confirm");
    let response = client
        .put(url)
        .send()
        .await
        .map_err(log_message_and_return(
            "Can't mark as confirmed on UDA",
            ConnectionFailed,
        ))?;

    let status = response.status();
    if !status.is_success() {
        return match status {
            StatusCode::NOT_FOUND => Err(LackOfPermissions), // If the user is not authorized to confirm members, then we get a 404...
            _ => Err(ConnectionFailed),
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
            Box::pin(confirm_member_with_retry(client, base_url, id, false)).await
        } else {
            error!(
                "Member has been unconfirmed! NOT trying to confirm them back. [uda_url: {base_url}, id: {id}]"
            );
            Err(CartMarkMember)
        }
    } else if body.contains(marked_message.as_str()) {
        trace!("Member has been confirmed on UDA! [uda_url: {base_url}, id: {id}]");
        Ok(())
    } else {
        Err(LackOfPermissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[async_test]
    async fn should_confirm_member() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        let body = format!(
            r##"var new_row = $("<tr class=\'confirmed\' id=\'reg_1\'>\n<td><a href=\"/en/registrants/1\">1<\/a><\/td>\n<td>\n<span class=\'member_number js--toggle\' data-toggle-target=\'#member_number_form_1\' id=\'membership_number_1\'>ID #012048<\/span>\n<span class=\'is--hidden\' id=\'member_number_form_1\'>\n<form action=\"/en/organization_memberships/1/update_number\" accept-charset=\"UTF-8\" data-remote=\"true\" method=\"post\"><input name=\"utf8\" type=\"hidden\" value=\"&#x2713;\" autocomplete=\"off\" /><input type=\"hidden\" name=\"_method\" value=\"put\" autocomplete=\"off\" /><input type=\"hidden\" name=\"authenticity_token\" value=\"fCnx1Z3o3n1jCeFbXxvniRDDcGt5wdQPNad8KQalzWw0qE3N56Q39nfPpoBG5fPXtu6RaSrDdUAvIkgOzCa5ug\" autocomplete=\"off\" /><input type=\"text\" name=\"membership_number\" id=\"membership_number\" value=\"012048\" />\n<input type=\"submit\" name=\"commit\" value=\"Update Membership #\" class=\"button tiny\" data-disable-with=\"Update Membership #\" />\n<\/form><\/span>\n<\/td>\n<td>François<\/td>\n<td>WURMSER<\/td>\n<td>34<\/td>\n<td>1985-03-20<\/td>\n<td>LA RICHE<\/td>\n<td>Indre-et-Loire<\/td>\n<td>France<\/td>\n<td>Roule Ta Bille<\/td>\n<td>\ntrue\n<\/td>\n<td>\nManually Confirmed\n<br>\n<a data-remote=\"true\" rel=\"nofollow\" data-method=\"put\" href=\"/en/organization_memberships/{id}/toggle_confirm\">Mark as unconfirmed<\/a>\n<\/td>\n<\/tr>\n")
old_row = $("#reg_1")
old_row.replaceWith(new_row)
new_row.effect("highlight", {{}}, 3000);
console.log("Updated 1"");"##
        );

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap();
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_lack_of_permissions() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        let body = "<div id='main'>
<p class='alert_flash'>You are not authorized to perform this action.</p>

I'm sorry, but the page you are searching for was not found.

</div>";

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .respond_with(ResponseTemplate::new(404).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert_eq!(LackOfPermissions, error);
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_connection_failed() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert_eq!(ConnectionFailed, error);
    }

    #[async_test]
    async fn should_fail_to_confirm_member_when_no_body() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("PUT"))
            .and(path(format!(
                "/en/organization_memberships/{id}/toggle_confirm"
            )))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert_eq!(LackOfPermissions, error);
    }

    #[async_test]
    async fn should_fail_to_confirm_member_and_not_retry_twice() {
        let id = 10_u16;

        let mock_server = MockServer::start().await;
        let client = Client::new();

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
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let error = confirm_member(&client, &mock_server.uri(), id)
            .await
            .unwrap_err();
        assert_eq!(CartMarkMember, error);
    }
}
