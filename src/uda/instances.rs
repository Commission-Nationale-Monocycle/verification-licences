use crate::database;
use crate::database::error::DatabaseError;
use crate::error::Result;
use crate::uda::configuration::Configuration;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dto::uda_instance::Instance;
use reqwest::Client;

/// Retrieve a list of all UDA instances.
pub async fn retrieve_uda_instances(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
    client: &Client,
    configuration: &Configuration,
) -> Result<Vec<Instance>> {
    let instances = uda_connector::instances::retrieve_uda_instances(
        &client,
        configuration.instances_list_url(),
    )
    .await?;

    let mut connection = pool.get().map_err(DatabaseError::from)?;
    database::dao::uda_instance::replace_all(&mut connection, &instances)?;

    Ok(instances)
}

#[cfg(test)]
pub(crate) mod tests {
    mod retrieve_uda_instances {
        use crate::database::with_temp_database;
        use crate::error::ApplicationError::UdaConnector;
        use crate::tools::web::build_client;
        use crate::uda::configuration::Configuration;
        use crate::uda::instances::retrieve_uda_instances;
        use diesel::SqliteConnection;
        use diesel::r2d2::ConnectionManager;
        use r2d2::Pool;
        use reqwest::header::LOCATION;
        use rocket::tokio::runtime::Runtime;
        use uda_connector::instances::{BODY, get_expected_instances};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test]
        fn success() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("tenants"))
                    .respond_with(ResponseTemplate::new(200).set_body_string(BODY))
                    .mount(&mock_server)
                    .await;

                let client = build_client().unwrap();
                let instances_list_url = format!("{}/tenants?locale=en", mock_server.uri());
                let configuration = Configuration::new(instances_list_url);
                let instances = retrieve_uda_instances(&pool, &client, &configuration)
                    .await
                    .unwrap();

                assert_eq!(get_expected_instances(), instances);
            }

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_connection_failed() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
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
                let error = retrieve_uda_instances(&pool, &client, &configuration)
                    .await
                    .unwrap_err();

                println!("{error:?}");

                assert!(matches!(
                    error,
                    UdaConnector(uda_connector::error::UdaError::ConnectionFailed)
                ));
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_cant_read_page_content() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let mock_server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("tenants"))
                    .respond_with(ResponseTemplate::new(500))
                    .mount(&mock_server)
                    .await;

                let client = build_client().unwrap();
                let instances_list_url = format!("{}/tenants?locale=en", mock_server.uri());
                let configuration = Configuration::new(instances_list_url);
                let error = retrieve_uda_instances(&pool, &client, &configuration)
                    .await
                    .unwrap_err();

                assert!(matches!(
                    error,
                    UdaConnector(uda_connector::error::UdaError::CantReadPageContent)
                ));
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }
}
