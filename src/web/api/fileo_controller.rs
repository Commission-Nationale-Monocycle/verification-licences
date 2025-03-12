use crate::member::config::MembershipsProviderConfig;
use crate::member::download::{build_client, login_to_fileo};
use crate::web::credentials::{Credentials, CredentialsStorage};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::Mutex;
use uuid::Uuid;

/// Try and log a user onto Fileo app.
/// If the login operation successes,
/// then a new UUID is created and credentials are stored with this UUID.
/// The UUID is returned to the caller, so that it is their new access token.
#[post("/fileo/login", format = "application/json", data = "<credentials>")]
pub async fn login(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    credentials_storage: &State<Mutex<CredentialsStorage>>,
    credentials: Json<Credentials>,
) -> Result<(Status, String), Status> {
    let client = build_client();
    if let Ok(client) = client {
        let host = memberships_provider_config.inner().host();
        let credentials = credentials.into_inner();
        match login_to_fileo(&client, host, &credentials).await {
            Ok(_) => {
                let mut mutex = credentials_storage
                    .lock()
                    .map_err(|_| Status::InternalServerError)?;
                let uuid = Uuid::new_v4().to_string();
                (*mutex).store(uuid.clone(), credentials);
                Ok((Status::Ok, uuid))
            }
            Err(_) => Err(Status::Unauthorized),
        }
    } else {
        Err(Status::InternalServerError)
    }
}
