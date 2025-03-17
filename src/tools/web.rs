use crate::tools::log_message_and_return;
use crate::web::error::WebError;
use crate::web::error::WebError::CantCreateClient;
use reqwest::Client;

pub fn build_client() -> Result<Client, WebError> {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .map_err(log_message_and_return(
            "Can't build HTTP client.",
            CantCreateClient,
        ))
}
