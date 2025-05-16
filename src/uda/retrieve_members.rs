use crate::error::{ApplicationError, Result};
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::uda::error::UdaError::{MalformedXlsFile, OrganizationMembershipsAccessFailed};
use crate::web::error::WebError::LackOfPermissions;
use calamine::{
    Data, RangeDeserializer, RangeDeserializerBuilder, Reader, Xls, open_workbook_from_rs,
};
use dto::uda_member::UdaMember;
use reqwest::Client;
use std::io::Cursor;
use uda_connector::imported_uda_member::ImportedUdaMember;

/// Retrieve members from UDA's organisation membership page.
pub async fn retrieve_members(client: &Client, base_url: &str) -> Result<Vec<UdaMember>> {
    let url = format!("{base_url}/en/organization_memberships/export.xls");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(log_error_and_return(OrganizationMembershipsAccessFailed))?;

    let status = response.status();
    if status.is_success() {
        let body = response.bytes().await.map_err(log_message_and_return(
            "Can't read organization_memberships content",
            OrganizationMembershipsAccessFailed,
        ))?;

        retrieve_imported_members_from_xls(Cursor::new(body)).map(|imported_members| {
            imported_members
                .into_iter()
                .map(|imported_member| imported_member.into())
                .collect()
        })
    } else if status.as_u16() == 401 {
        error!("Can't access organization_memberships page. Lack of permissions?");
        Err(ApplicationError::from(LackOfPermissions))
    } else {
        error!(
            "Can't reach organization_memberships page: {:?}",
            response.status()
        );
        Err(OrganizationMembershipsAccessFailed)?
    }
}

fn retrieve_imported_members_from_xls<T: AsRef<[u8]>>(
    cursor: Cursor<T>,
) -> Result<Vec<ImportedUdaMember>> {
    let mut workbook: Xls<_> =
        open_workbook_from_rs(cursor).map_err(log_error_and_return(MalformedXlsFile))?;
    let sheets = workbook.sheet_names();
    let first_sheet = sheets.first();
    let worksheet_name = first_sheet.ok_or(MalformedXlsFile)?;
    let range = workbook
        .worksheet_range(worksheet_name)
        .map_err(log_message_and_return(
            "Can't read organization_memberships content",
            MalformedXlsFile,
        ))?;
    let deserializer: RangeDeserializer<'_, Data, ImportedUdaMember> =
        RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&range)
            .map_err(log_message_and_return(
                "Can't read organization_memberships content",
                MalformedXlsFile,
            ))?;

    let members = deserializer
        .flat_map(|result| match result {
            Ok(member) => {
                match member.id() {
                    0..2000 => Some(member),
                    _ => None, // IDs over 2000 relate to non-competitors: they don't require a membership.
                }
            }
            Err(error) => {
                warn!("Can't deserialize UDA member. Ignoring. {:?}", error);
                None
            }
        })
        .collect();

    Ok(members)
}

#[cfg(test)]
pub mod tests {
    use dto::uda_member::UdaMember;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn get_test_file_content() -> Vec<u8> {
        std::fs::read("test/resources/uda_members.xls").unwrap()
    }

    fn get_expected_member() -> Vec<UdaMember> {
        vec![
            UdaMember::new(
                1,
                Some("123456".to_owned()),
                "Jon".to_owned(),
                "Doe".to_owned(),
                "jon.doe@email.com".to_owned(),
                Some("Le club de test".to_owned()),
                true,
            ),
            UdaMember::new(
                2,
                Some("654321".to_owned()),
                "Jonette".to_owned(),
                "Snow".to_owned(),
                "jonette.snow@email.com".to_owned(),
                None,
                false,
            ),
            UdaMember::new(
                1999,
                Some("456789".to_owned()),
                "Kris".to_owned(),
                "Holm".to_owned(),
                "kris.holm@email.com".to_owned(),
                Some("KH Team".to_owned()),
                true,
            ),
        ]
    }

    pub async fn setup_member_retrieval(mock_server: &MockServer) -> Vec<UdaMember> {
        let body = get_test_file_content();

        Mock::given(method("GET"))
            .and(path("/en/organization_memberships/export.xls"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
            .mount(mock_server)
            .await;

        get_expected_member()
    }

    mod retrieve_members {
        use crate::error::ApplicationError::{Uda, Web};
        use crate::tools::web::build_client;
        use crate::uda::error::UdaError;
        use crate::uda::retrieve_members::retrieve_members;
        use crate::uda::retrieve_members::tests::setup_member_retrieval;
        use crate::web::error::WebError::LackOfPermissions;
        use UdaError::OrganizationMembershipsAccessFailed;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            let expected_result = setup_member_retrieval(&mock_server).await;
            let client = build_client().unwrap();
            let result = retrieve_members(&client, &mock_server.uri()).await.unwrap();
            assert_eq!(expected_result, result);
        }

        #[async_test]
        async fn fail_when_unreachable() {
            let mock_server = MockServer::start().await;
            let client = build_client().unwrap();
            Mock::given(method("GET"))
                .and(path("en/organization_memberships/export.xls"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&mock_server)
                .await;

            let error = retrieve_members(&client, &mock_server.uri())
                .await
                .unwrap_err();
            assert!(matches!(error, Uda(OrganizationMembershipsAccessFailed)));
        }

        #[async_test]
        async fn fail_when_lack_of_permissions() {
            let mock_server = MockServer::start().await;
            let client = build_client().unwrap();
            Mock::given(method("GET"))
                .and(path("en/organization_memberships/export.xls"))
                .respond_with(ResponseTemplate::new(401))
                .mount(&mock_server)
                .await;

            let error = retrieve_members(&client, &mock_server.uri())
                .await
                .unwrap_err();
            assert!(matches!(error, Web(LackOfPermissions)));
        }
    }

    mod retrieve_imported_members_from_xls {
        use crate::error::ApplicationError::Uda;
        use crate::uda::error::UdaError;
        use crate::uda::retrieve_members::retrieve_imported_members_from_xls;
        use crate::uda::retrieve_members::tests::get_test_file_content;
        use UdaError::MalformedXlsFile;
        use std::io::Cursor;
        use uda_connector::imported_uda_member::ImportedUdaMember;

        fn get_expected_imported_members() -> Vec<ImportedUdaMember> {
            vec![
                ImportedUdaMember::new(
                    1,
                    Some("123456".to_owned()),
                    None,
                    "Jon".to_owned(),
                    "Doe".to_owned(),
                    "01.02.1983".to_owned(),
                    "42, Le Village".to_owned(),
                    "Cartuin".to_owned(),
                    Some("Creuse".to_owned()),
                    "23340".to_owned(),
                    "FR".to_owned(),
                    Some("0123456789".to_owned()),
                    "jon.doe@email.com".to_owned(),
                    Some("Le club de test".to_owned()),
                    true,
                ),
                ImportedUdaMember::new(
                    2,
                    Some("654321".to_owned()),
                    None,
                    "Jonette".to_owned(),
                    "Snow".to_owned(),
                    "12.11.1990".to_owned(),
                    "1337, Là-bas".to_owned(),
                    "Setif".to_owned(),
                    Some("Sétif".to_owned()),
                    "19046".to_owned(),
                    "DZ".to_owned(),
                    Some("987654321".to_owned()),
                    "jonette.snow@email.com".to_owned(),
                    None,
                    false,
                ),
                ImportedUdaMember::new(
                    1999,
                    Some("456789".to_owned()),
                    None,
                    "Kris".to_owned(),
                    "Holm".to_owned(),
                    "10.08.1975".to_owned(),
                    "57, The Mountain".to_owned(),
                    "Everest".to_owned(),
                    Some("Canada".to_owned()),
                    "78945".to_owned(),
                    "CA".to_owned(),
                    None,
                    "kris.holm@email.com".to_owned(),
                    Some("KH Team".to_owned()),
                    true,
                ),
            ]
        }

        #[test]
        fn success() {
            let content = get_test_file_content();
            let members = retrieve_imported_members_from_xls(Cursor::new(content)).unwrap();
            assert_eq!(get_expected_imported_members(), members)
        }

        #[test]
        fn ignore_member_when_missing_field() {
            let content = std::fs::read("test/resources/uda_members_1_invalid.xls").unwrap();
            let cursor = Cursor::new(content);
            let members = retrieve_imported_members_from_xls(cursor).unwrap();
            assert_eq!(
                vec![ImportedUdaMember::new(
                    1,
                    Some("123456".to_owned()),
                    None,
                    "Jon".to_owned(),
                    "Doe".to_owned(),
                    "01.02.1983".to_owned(),
                    "42, Le Village".to_owned(),
                    "Cartuin".to_owned(),
                    Some("Creuse".to_owned()),
                    "23340".to_owned(),
                    "FR".to_owned(),
                    Some("0123456789".to_owned()),
                    "jon.doe@email.com".to_owned(),
                    Some("Le club de test".to_owned()),
                    true,
                )],
                members
            );
        }

        #[test]
        fn fail_when_malformed_xls() {
            let error = retrieve_imported_members_from_xls(Cursor::new(""))
                .err()
                .unwrap();
            assert!(matches!(error, Uda(MalformedXlsFile)));
        }
    }
}
