use crate::error::{ApplicationError, Result};
use crate::tools::{log_error_and_return, log_message_and_return};
use crate::uda::error::UdaError::{MalformedXlsFile, OrganizationMembershipsAccessFailed};
use crate::uda::participant::ImportedParticipant;
use crate::web::error::WebError::LackOfPermissions;
use calamine::{
    Data, RangeDeserializer, RangeDeserializerBuilder, Reader, Xls, open_workbook_from_rs,
};
use dto::participant::Participant;
use reqwest::Client;
use std::io::Cursor;

/// Retrieve members from UDA's organisation membership page.
pub async fn retrieve_participants(client: &Client, base_url: &str) -> Result<Vec<Participant>> {
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

        retrieve_imported_participants_from_xls(Cursor::new(body)).map(|imported_participants| {
            imported_participants
                .into_iter()
                .map(|imported_participant| imported_participant.into())
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

fn retrieve_imported_participants_from_xls<T: AsRef<[u8]>>(
    cursor: Cursor<T>,
) -> Result<Vec<ImportedParticipant>> {
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
    let deserializer: RangeDeserializer<'_, Data, ImportedParticipant> =
        RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&range)
            .map_err(log_message_and_return(
                "Can't read organization_memberships content",
                MalformedXlsFile,
            ))?;

    let participants = deserializer
        .flat_map(|result| match result {
            Ok(participant) => Some(participant),
            Err(error) => {
                warn!("Can't deserialize participant. Ignoring. {:?}", error);
                None
            }
        })
        .collect();

    Ok(participants)
}

#[cfg(test)]
pub mod tests {
    use dto::participant::Participant;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn get_test_file_content() -> Vec<u8> {
        std::fs::read("test/resources/uda_participant.xls").unwrap()
    }

    fn get_expected_participants() -> Vec<Participant> {
        vec![
            Participant::new(
                1,
                Some("123456".to_owned()),
                "Jon".to_owned(),
                "Doe".to_owned(),
                "jon.doe@email.com".to_owned(),
                Some("Le club de test".to_owned()),
                true,
            ),
            Participant::new(
                2,
                Some("654321".to_owned()),
                "Jonette".to_owned(),
                "Snow".to_owned(),
                "jonette.snow@email.com".to_owned(),
                None,
                false,
            ),
        ]
    }

    pub async fn setup_participant_retrieval(mock_server: &MockServer) -> Vec<Participant> {
        let body = get_test_file_content();

        Mock::given(method("GET"))
            .and(path("/en/organization_memberships/export.xls"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
            .mount(mock_server)
            .await;

        get_expected_participants()
    }

    mod retrieve_participants {
        use crate::error::ApplicationError::{Uda, Web};
        use crate::tools::web::build_client;
        use crate::uda::error::UdaError;
        use crate::uda::retrieve_members::retrieve_participants;
        use crate::uda::retrieve_members::tests::setup_participant_retrieval;
        use crate::web::error::WebError::LackOfPermissions;
        use UdaError::OrganizationMembershipsAccessFailed;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[async_test]
        async fn success() {
            let mock_server = MockServer::start().await;
            let expected_result = setup_participant_retrieval(&mock_server).await;
            let client = build_client().unwrap();
            let result = retrieve_participants(&client, &mock_server.uri())
                .await
                .unwrap();
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

            let error = retrieve_participants(&client, &mock_server.uri())
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

            let error = retrieve_participants(&client, &mock_server.uri())
                .await
                .unwrap_err();
            assert!(matches!(error, Web(LackOfPermissions)));
        }
    }

    mod retrieve_imported_participants_from_xls {
        use crate::error::ApplicationError::Uda;
        use crate::uda::error::UdaError;
        use crate::uda::participant::ImportedParticipant;
        use crate::uda::retrieve_members::retrieve_imported_participants_from_xls;
        use crate::uda::retrieve_members::tests::get_test_file_content;
        use UdaError::MalformedXlsFile;
        use std::io::Cursor;

        fn get_expected_imported_participants() -> Vec<ImportedParticipant> {
            vec![
                ImportedParticipant::new(
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
                ImportedParticipant::new(
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
            ]
        }

        #[test]
        fn success() {
            let content = get_test_file_content();
            let participants =
                retrieve_imported_participants_from_xls(Cursor::new(content)).unwrap();
            assert_eq!(get_expected_imported_participants(), participants)
        }

        #[test]
        fn ignore_participant_when_missing_field() {
            let content = std::fs::read("test/resources/uda_participant_1_invalid.xls").unwrap();
            let cursor = Cursor::new(content);
            let participants = retrieve_imported_participants_from_xls(cursor).unwrap();
            assert_eq!(
                vec![ImportedParticipant::new(
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
                participants
            );
        }

        #[test]
        fn fail_when_malformed_xls() {
            let error = retrieve_imported_participants_from_xls(Cursor::new(""))
                .err()
                .unwrap();
            assert!(matches!(error, Uda(MalformedXlsFile)));
        }
    }
}
