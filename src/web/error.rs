use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum WebError {
    #[error("Client couldn't be created.")]
    CantCreateClient,
    #[error("The credentials that have been passed seem to not match any known credentials.")]
    WrongCredentials,
    #[error("The connection to the other server failed.")]
    ConnectionFailed,
    #[error("The page that has been downloaded doesn't provide any content.")]
    CantReadPageContent,
    #[error(
        "Although the credentials are OK, the user doesn't have permissions to execute the operation."
    )]
    LackOfPermissions,
    #[error("The requested page or file has not been found.")]
    NotFound,
}
