use crate::error::ApplicationError::Web;
use crate::error::Result;
use crate::tools::log_error_and_return;
use crate::uda::configuration::Configuration;
use crate::uda::error::UdaError;
use crate::web::error::WebError::{CantReadPageContent, ConnectionFailed};
use dto::uda::Instance;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

/// Retrieve a list of all UDA instances.
#[allow(dead_code)]
pub async fn retrieve_uda_instances(
    client: &Client,
    configuration: &Configuration,
) -> Result<Vec<Instance>> {
    let response = client
        .get(configuration.instances_list_url())
        .send()
        .await
        .map_err(log_error_and_return(Web(ConnectionFailed)))?;
    let status = response.status();
    if !status.is_success() {
        Err(Web(CantReadPageContent))?;
    }

    let body = response
        .text()
        .await
        .map_err(log_error_and_return(Web(CantReadPageContent)))?;

    get_uda_instances_from_html(&body)
}

fn get_uda_instances_from_html(body: &str) -> Result<Vec<Instance>> {
    let selector = Selector::parse(r"tr").map_err(UdaError::from)?;
    let document = Html::parse_document(body);

    let rows = document.select(&selector);

    Ok(rows.flat_map(|x| get_uda_instance_from_row(&x)).collect())
}

fn get_uda_instance_from_row(row: &ElementRef) -> Option<Instance> {
    let selector = Selector::parse("td").ok()?;
    let cells = row.select(&selector).collect::<Vec<_>>();

    let (subdomain_cell, name_cell) = match cells[..] {
        [subdomain_cell, name_cell, _creation_date] => (subdomain_cell, name_cell),
        _ => {
            warn!("Ignoring instance because wrongly formatted [row: {row:?}]");
            return None;
        }
    };

    let selector = Selector::parse("a").ok()?;
    let link_element = subdomain_cell.select(&selector).next()?;
    let link = link_element.attr("href")?;
    let slug = link_element.text().next()?;
    let name = name_cell.text().next()?;

    Some(Instance::new(
        slug.to_owned(),
        name.to_owned(),
        link.to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::uda::instances::Instance;

    const BODY: &str = r##"<html class=""><head></head><body><div id="container"><div id="main"><h1>Existing Conventions</h1><table><thead><tr><th>Subdomain</th><th>Description</th><th>Created At</th></tr></thead><tbody><tr><td><a href="https://mt-ventoux-2023.reg.unicycling-software.com">mt-ventoux-2023</a></td><td>Mount Ventoux Unicycle Challenge</td><td>Wed, 03 May 2023 15:12:52 -0500</td></tr><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table><hr><a class="button" href="/tenants/new?locale=en">New Convention</a></div></div></body></html>"##;
    const MALFORMED_BODY: &str = r##"<html class=""><head></head><body><div id="container"><div id="main"><h1>Existing Conventions</h1><table><thead><tr><th>Subdomain</th><th>Description</th><th>Created At</th></tr></thead><tbody><tr><td><a href="https://mt-ventoux-2023.reg.unicycling-software.com">mt-ventoux-2023</a></td><td>Mount Ventoux Unicycle Challenge</td><td>Wed, 03 May 2023 15:12:52 -0500</td></tr><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td>CFM 2025</td></tr></tbody></table><hr><a class="button" href="/tenants/new?locale=en">New Convention</a></div></div></body></html>"##;

    fn get_expected_instances() -> Vec<Instance> {
        vec![
            Instance::new(
                "mt-ventoux-2023".to_owned(),
                "Mount Ventoux Unicycle Challenge".to_owned(),
                "https://mt-ventoux-2023.reg.unicycling-software.com".to_owned(),
            ),
            Instance::new(
                "cfm2025".to_owned(),
                "CFM 2025".to_owned(),
                "https://cfm2025.reg.unicycling-software.com".to_owned(),
            ),
        ]
    }

    mod retrieve_uda_instances {
        use crate::error::ApplicationError::Web;
        use crate::tools::web::build_client;
        use crate::uda::configuration::Configuration;
        use crate::uda::instances::retrieve_uda_instances;
        use crate::uda::instances::tests::{BODY, get_expected_instances};
        use crate::web::error::WebError::{CantReadPageContent, ConnectionFailed};
        use reqwest::header::LOCATION;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("tenants"))
                .respond_with(ResponseTemplate::new(200).set_body_string(BODY))
                .mount(&mock_server)
                .await;

            let client = build_client().unwrap();
            let instances_list_url = format!("{}/tenants?locale=en", mock_server.uri());
            let configuration = Configuration::new(instances_list_url);
            let instances = retrieve_uda_instances(&client, &configuration)
                .await
                .unwrap();

            assert_eq!(get_expected_instances(), instances);
        }

        #[async_test]
        async fn fail_when_connection_failed() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("tenants"))
                .respond_with(
                    ResponseTemplate::new(301).append_header(LOCATION, "/tenants?locale=en"),
                )
                .mount(&mock_server)
                .await;

            let client = build_client().unwrap();
            let instances_list_url = format!("{}/tenants?locale=en", mock_server.uri());
            let configuration = Configuration::new(instances_list_url);
            let error = retrieve_uda_instances(&client, &configuration)
                .await
                .unwrap_err();

            println!("{error:?}");

            assert!(matches!(error, Web(ConnectionFailed)));
        }

        #[async_test]
        async fn fail_when_cant_read_page_content() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("tenants"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&mock_server)
                .await;

            let client = build_client().unwrap();
            let instances_list_url = format!("{}/tenants?locale=en", mock_server.uri());
            let configuration = Configuration::new(instances_list_url);
            let error = retrieve_uda_instances(&client, &configuration)
                .await
                .unwrap_err();

            assert!(matches!(error, Web(CantReadPageContent)));
        }
    }

    mod get_uda_instances_from_html {
        use crate::uda::instances::tests::{BODY, MALFORMED_BODY, get_expected_instances};
        use crate::uda::instances::{Instance, get_uda_instances_from_html};

        #[test]
        fn success() {
            let instances = get_uda_instances_from_html(BODY).unwrap();
            assert_eq!(get_expected_instances(), instances);
        }

        #[test]
        fn get_only_one_out_of_two_when_one_malformed() {
            let expected_instance = Instance::new(
                "mt-ventoux-2023".to_owned(),
                "Mount Ventoux Unicycle Challenge".to_owned(),
                "https://mt-ventoux-2023.reg.unicycling-software.com".to_owned(),
            );

            let instances = get_uda_instances_from_html(MALFORMED_BODY).unwrap();
            assert_eq!(vec![expected_instance], instances);
        }
    }

    mod get_uda_instance_from_row {
        use crate::uda::instances::get_uda_instance_from_row;
        use scraper::{Html, Selector};

        #[test]
        fn success() {
            let body = r#"<html><head></head><body><table><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            let instance = get_uda_instance_from_row(&row).unwrap();
            assert_eq!("cfm2025", instance.slug());
            assert_eq!("CFM 2025", instance.name());
            assert_eq!(
                "https://cfm2025.reg.unicycling-software.com",
                instance.url()
            );
        }

        #[test]
        fn fail_when_missing_cells() {
            let body = r#"<html><head></head><body><table><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td>CFM 2025</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }

        #[test]
        fn fail_when_too_many_cells() {
            let body = r#"<html><head></head><body><table><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td><td>Just another cell</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }

        #[test]
        fn fail_when_no_link_element() {
            let body = r#"<html><head></head><body><table><tr><td>cfm2025</td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }

        #[test]
        fn fail_when_no_href_attr() {
            let body = r#"<html><head></head><body><table><tr><td><a>cfm2025</a></td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }

        #[test]
        fn fail_when_no_name() {
            let body = r#"<html><head></head><body><table><tr><td><a href="https://cfm2025.reg.unicycling-software.com">cfm2025</a></td><td></td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }

        #[test]
        fn fail_when_no_slug() {
            let body = r#"<html><head></head><body><table><tr><td><a href="https://cfm2025.reg.unicycling-software.com"></a></td><td>CFM 2025</td><td>Mon, 27 Jan 2025 13:13:52 -0600</td></tr></tbody></table></body></html>"#;
            let selector = Selector::parse(r"tr").unwrap();
            let document = Html::parse_document(body);

            let row = document.select(&selector).next().unwrap();
            assert!(get_uda_instance_from_row(&row).is_none());
        }
    }
}
