use crate::member::config::MembershipsProviderConfig;
use crate::member::download::{build_client, login_to_fileo};
use crate::web::credentials::Credentials;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;

#[post("/fileo/login", format = "application/json", data = "<credentials>")]
pub async fn login(
    memberships_provider_config: &State<MembershipsProviderConfig>,
    credentials: Json<Credentials>,
) -> Status {
    let client = build_client();
    if let Ok(client) = client {
        let host = memberships_provider_config.inner().host();
        match login_to_fileo(&client, host, &credentials).await {
            Ok(_) => Status::NoContent,
            Err(_) => Status::Unauthorized,
        }
    } else {
        Status::InternalServerError
    }
}
