use crate::tools::error::Error::{CantAccessOrganizationMemberships, LackOfPermissions};
use crate::tools::error::Result;
use crate::tools::{log_error_and_return, log_message_and_return};
use dto::member_to_check::MemberToCheck;
use reqwest::Client;
use scraper::{ElementRef, Html, Node, Selector};

#[allow(dead_code)]
pub async fn retrieve_members(client: &Client, base_url: &str) -> Result<Vec<MemberToCheck>> {
    let url = format!("{base_url}/en/organization_memberships");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(log_error_and_return(CantAccessOrganizationMemberships))?;

    let status = response.status();
    if status.is_success() {
        let body = response.text().await.map_err(log_message_and_return(
            "Can't read organization_memberships content",
            CantAccessOrganizationMemberships,
        ))?;
        if body.contains("Unicycling Society/Federation Membership Management") {
            retrieve_members_from_html(&body)
        } else {
            error!("Can't access organization_memberships page. Lack of permissions?");
            Err(LackOfPermissions)
        }
    } else {
        error!(
            "Can't reach organization_memberships page: {:?}",
            response.status()
        );
        Err(CantAccessOrganizationMemberships)
    }
}

fn retrieve_members_from_html(body: &str) -> Result<Vec<MemberToCheck>> {
    let selector = Selector::parse("[id^=reg_]")?;
    let document = Html::parse_document(body);
    Ok(document
        .select(&selector)
        .flat_map(extract_member_from_row)
        .collect())
}

/// From a row, extract a MemberToCheck.
/// If the row is malformed (= does not match with what's expected), then ignore the row.
///
/// The implementation is ugly, but it's difficult to do otherwise because of Scraper's lib.
/// Ideally, we'd like to extract each value extraction, but we can't do so because `NodeRef` is inaccessible.
fn extract_member_from_row(row: ElementRef) -> Option<MemberToCheck> {
    let cells = row
        .children()
        .filter(|child| match child.value() {
            Node::Element(_) => true,
            // Ignoring all the line feeds that don't matter here.
            _ => false,
        })
        .collect::<Vec<_>>();

    if cells.len() < 4 {
        warn!(
            "Member row is too short. Should have at least 4 cells. Ignoring. [row: {}]",
            row.html()
        );
        return None;
    }

    let id_span = cells[1]
        .children()
        .find(|child| matches!(child.value(), Node::Element(_)));
    if id_span.is_none() {
        warn!(
            "Missing id span for member. Ignoring. [row: {}]",
            row.html()
        );
        return None;
    }

    let id_text = id_span.unwrap().first_child();
    if id_text.is_none() {
        warn!("Wrong structure for id. Ignoring. [row: {}]", row.html());
        return None;
    }

    let id_value = id_text.unwrap().value().as_text();
    if id_value.is_none() {
        warn!(
            "ID span of member doesn't contain only text. Ignoring. [row: {}]",
            row.html()
        );
        return None;
    }

    let id_text = id_value.unwrap();
    let id = if id_text.text.to_string() == "set number" {
        None
    } else {
        Some(id_text.to_string().trim_start_matches("ID #").to_owned())
    };

    let first_name = cells[2]
        .first_child()
        .or_else(|| {
            warn!(
                "Missing first name for member. Ignoring. [row: {}]",
                row.html()
            );
            None
        })?
        .value()
        .as_text()
        .map(|s| s.to_string())
        .or_else(|| {
            warn!(
                "First name cells contains more than text. Ignoring. [row: {}]",
                row.html()
            );
            None
        })?;
    let last_name = cells[3]
        .first_child()
        .or_else(|| {
            warn!(
                "Missing last name for member. Ignoring. [row: {}]",
                row.html()
            );
            None
        })?
        .value()
        .as_text()
        .map(|s| s.to_string())
        .or_else(|| {
            warn!(
                "Missing last name for member. Ignoring. [row: {}]",
                row.html()
            );
            None
        })?;

    Some(MemberToCheck::new(id, first_name, last_name))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tools::web::build_client;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    pub async fn setup_members_to_check_retrieval(mock_server: &MockServer) -> Vec<MemberToCheck> {
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
            .mount(mock_server)
            .await;

        vec![
            MemberToCheck::new(
                Some(expected_id_1.to_owned()),
                expected_first_name_1.to_owned(),
                expected_name_1.to_owned(),
            ),
            MemberToCheck::new(
                Some(expected_id_2.to_owned()),
                expected_first_name_2.to_owned(),
                expected_name_2.to_owned(),
            ),
        ]
    }

    // region retrieve_members
    #[async_test]
    async fn should_retrieve_members() {
        let mock_server = MockServer::start().await;
        let expected_result = setup_members_to_check_retrieval(&mock_server).await;
        let client = build_client().unwrap();
        let result = retrieve_members(&client, &mock_server.uri()).await.unwrap();
        assert_eq!(expected_result, result);
    }

    #[async_test]
    async fn should_fail_to_retrieve_members_when_unreachable() {
        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = retrieve_members(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert_eq!(CantAccessOrganizationMemberships, result);
    }

    #[async_test]
    async fn should_fail_to_retrieve_members_when_lack_of_permissions() {
        let body = "<html><body>You should log in to access this page.</body></html>";

        let mock_server = MockServer::start().await;
        let client = build_client().unwrap();
        Mock::given(method("GET"))
            .and(path("/en/organization_memberships"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let result = retrieve_members(&client, &mock_server.uri())
            .await
            .unwrap_err();
        assert_eq!(LackOfPermissions, result);
    }
    // endregion

    // region retrieve_members_from_html
    #[test]
    fn should_retrieve_members_from_html() {
        let expected_id_1 = "123456";
        let expected_first_name_1 = "Jon";
        let expected_name_1 = "DOE";
        let expected_id_2 = "654321";
        let expected_first_name_2 = "Jonette";
        let expected_name_2 = "Snow";
        let expected_result = vec![
            MemberToCheck::new(
                Some(expected_id_1.to_owned()),
                expected_first_name_1.to_owned(),
                expected_name_1.to_owned(),
            ),
            MemberToCheck::new(
                Some(expected_id_2.to_owned()),
                expected_first_name_2.to_owned(),
                expected_name_2.to_owned(),
            ),
        ];

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id_1}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name_1}</td><td>{expected_name_1}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr><tr class="even" id="reg_1" role="row"><td><a href="/fr/registrants/1">1</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_1" id="membership_number_1">ID #{expected_id_2}</span><span class="is--hidden" id="member_number_form_1"><form action="/fr/organization_memberships/1/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name_2}</td><td>{expected_name_2}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/1/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );

        let result = retrieve_members_from_html(&body).unwrap();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn should_retrieve_0_member_from_html_when_no_row() {
        let body = "<html><body><table></table></body></html>".to_string();

        let result = retrieve_members_from_html(&body).unwrap();
        assert!(
            result.is_empty(),
            "There should not be any member to check when no row [result: {result:?}]"
        );
    }
    // endregion

    // region extract_member_from_row
    #[test]
    fn should_extract_member_from_row() {
        let expected_id = "123456";
        let expected_first_name = "Jon";
        let expected_name = "DOE";
        let expected_result = MemberToCheck::new(
            Some(expected_id.to_owned()),
            expected_first_name.to_owned(),
            expected_name.to_owned(),
        );

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row).unwrap();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn should_fail_extract_member_from_row_when_too_short() {
        let expected_id = "123456";
        let expected_first_name = "Jon";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);

        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_extract_member_from_row_when_id_cell_contains_more_than_text() {
        let expected_id = "123456";
        let expected_first_name = "Jon";
        let expected_name = "DOE";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0"><p>ID #{expected_id}</p></span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_extract_member_from_row_when_id_cell_wrongly_structured() {
        let expected_first_name = "Jon";
        let expected_name = "DOE";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td>What are ya lookin' for, son?<span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_extract_member_from_row_when_id_cell_misses_span() {
        let expected_first_name = "Jon";
        let expected_name = "DOE";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_to_extract_member_from_row_when_missing_first_name() {
        let expected_id = "123456";
        let expected_name = "DOE";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td></td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_to_extract_member_from_row_when_first_name_contains_more_than_text() {
        let expected_id = "123456";
        let expected_name = "DOE";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td><p>Oops</p></td><td>{expected_name}</td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_to_extract_member_from_row_when_missing_name() {
        let expected_id = "123456";
        let expected_first_name = "Jon";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td></td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }

    #[test]
    fn should_fail_to_extract_member_from_row_when_name_contains_more_than_text() {
        let expected_id = "123456";
        let expected_first_name = "Jon";

        let body = format!(
            r##"<html><body><table><tr class="even" id="reg_0" role="row"><td><a href="/fr/registrants/0">0</a></td><td><span class="member_number js--toggle" data-toggle-target="#member_number_form_0" id="membership_number_0">ID #{expected_id}</span><span class="is--hidden" id="member_number_form_0"><form action="/fr/organization_memberships/0/update_number" accept-charset="UTF-8" data-remote="true" method="post"><input name="utf8" type="hidden" value="✓" autocomplete="off"><input type="hidden" name="_method" value="put" autocomplete="off"><input type="hidden" name="authenticity_token" value="pHkm6aZpTgLtAUdx3Nklm7nBsG5ECpEhKp9lB1_8YLjP9OwPhlEdYXdcrnDgAnue37U-8VOS6mJDWeaHqvgOag" autocomplete="off"><input type="text" name="membership_number" id="membership_number" value=""><input type="submit" name="commit" value="Update Membership #" class="button tiny" data-disable-with="Update Membership #"></form></span></td><td>{expected_first_name}</td><td><p>Oops</p></td><td>29</td><td>1990-05-06</td><td>Setif</td><td>Sétif</td><td>Algérie</td><td></td><td>false<br></td><td><a data-remote="true" rel="nofollow" data-method="put" href="/fr/organization_memberships/0/toggle_confirm">Mark as confirmed</a></td></tr></table></body></html>"##
        );
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[id^=reg_]").unwrap();
        let row = document.select(&selector).next().unwrap();

        let result = extract_member_from_row(row);
        assert_eq!(None, result);
    }
    // endregion
}
