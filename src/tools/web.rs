use crate::tools::error::Error::CantCreateClient;
use crate::tools::error::Result;
use crate::tools::log_message_and_return;
use reqwest::Client;

pub fn build_client() -> Result<Client> {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .map_err(log_message_and_return(
            "Can't build HTTP client.",
            CantCreateClient,
        ))
}
