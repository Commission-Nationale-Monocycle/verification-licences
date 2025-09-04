use crate::tools::email::Error::{
    CantConnectToSmtpServer, CantSendMessage, MissingEmailSenderAddress, MissingEmailSenderName,
    MissingSmtpLogin, MissingSmtpPassword,
};
use crate::tools::env_args::{retrieve_arg_value, retrieve_expected_arg_value};
use crate::tools::log_message_and_return;
use mail_send::SmtpClientBuilder;
use mail_send::mail_builder::MessageBuilder;
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

const EMAIL_SENDER_NAME_ARG: &str = "--email-sender-name";
const EMAIL_SENDER_ADDRESS_ARG: &str = "--email-sender-address";
const REPLY_TO_ARG: &str = "--reply-to";
const SMTP_SERVER_ARG: &str = "--smtp-server";
const SMTP_PORT_ARG: &str = "--smtp-port";
const SMTP_LOGIN_ARG: &str = "--smtp-login";
const SMTP_PASSWORD_ARG: &str = "--smtp-password";
const DEFAULT_SMTP_SERVER: &str = "smtp.gmail.com";
const DEFAULT_SMTP_PORT: u16 = 587;

pub async fn send_email(recipients: &[&str], subject: &str, text_body: &str) -> Result<()> {
    let message = create_message(recipients, subject, text_body)?;
    create_smtp_client_and_send_email(message).await
}

async fn create_smtp_client_and_send_email(message: MessageBuilder<'_>) -> Result<()> {
    let smtp_server = retrieve_smtp_server();
    let smtp_port = retrieve_smtp_port();
    let smtp_login = retrieve_smtp_login()?;
    let smtp_password = retrieve_smtp_password()?;
    let smtp_client = SmtpClientBuilder::new(smtp_server, smtp_port)
        .implicit_tls(false)
        .credentials((smtp_login, smtp_password))
        .connect()
        .await;

    smtp_client
        .map_err(log_message_and_return(
            "Couldn't connect to SMTP server",
            CantConnectToSmtpServer,
        ))?
        .send(message)
        .await
        .map_err(log_message_and_return(
            "Couldn't send message",
            CantSendMessage,
        ))
}

fn create_message<'a>(
    recipients: &'a [&str],
    subject: &'a str,
    text_body: &'a str,
) -> Result<MessageBuilder<'a>> {
    let sender_name = retrieve_email_sender_name()?;
    let sender_address = retrieve_email_sender_address()?;
    let reply_to_address = retrieve_reply_to().unwrap_or_else(|| sender_address.clone());

    Ok(MessageBuilder::new()
        .from((sender_name, sender_address))
        .reply_to(reply_to_address.clone())
        .to(reply_to_address)
        .bcc(Vec::from(recipients))
        .subject(subject)
        .text_body(text_body)
        .html_body(text_body))
}

// region Retrieve args
fn retrieve_smtp_server() -> String {
    retrieve_arg_value(SMTP_SERVER_ARG).unwrap_or(DEFAULT_SMTP_SERVER.to_owned())
}
fn retrieve_smtp_port() -> u16 {
    retrieve_arg_value(SMTP_PORT_ARG)
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(DEFAULT_SMTP_PORT)
}
fn retrieve_smtp_login() -> Result<String> {
    retrieve_expected_arg_value(SMTP_LOGIN_ARG, MissingSmtpLogin)
}

fn retrieve_smtp_password() -> Result<String> {
    retrieve_expected_arg_value(SMTP_PASSWORD_ARG, MissingSmtpPassword)
}

fn retrieve_email_sender_name() -> Result<String> {
    retrieve_expected_arg_value(EMAIL_SENDER_NAME_ARG, MissingEmailSenderName)
}

fn retrieve_email_sender_address() -> Result<String> {
    retrieve_expected_arg_value(EMAIL_SENDER_ADDRESS_ARG, MissingEmailSenderAddress)
}

fn retrieve_reply_to() -> Option<String> {
    retrieve_arg_value(REPLY_TO_ARG)
}
// endregion

#[derive(Debug, PartialEq, Error)]
pub enum Error {
    #[error("Missing email sender name")]
    MissingEmailSenderName,
    #[error("Missing email sender address")]
    MissingEmailSenderAddress,
    #[error("Missing SMTP login")]
    MissingSmtpLogin,
    #[error("Missing SMTP password")]
    MissingSmtpPassword,
    #[error("Can't connect to SMTP server")]
    CantConnectToSmtpServer,
    #[error("Can't send message")]
    CantSendMessage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::env_args::with_env_args;
    use mail_send::mail_builder::mime::BodyPart;
    use parameterized::{ide, parameterized};
    use rocket::futures::executor::block_on;

    ide!();

    const TEST_SMTP_SERVER: &str = "sandbox.smtp.mailtrap.io";
    const TEST_SMTP_PORT: u16 = 25;
    const TEST_EMAIL_SENDER_NAME: &str = "Sender";
    const TEST_EMAIL_SENDER_ADDRESS: &str = "sender@address.com";
    const TEST_REPLY_TO: &str = "sender+reply-to@address.com";
    const TEST_RECIPIENTS: &[&str] = &["recipient@address.com"];
    const TEST_SUBJECT: &str = "This is a subject";
    const TEST_TEXT_BODY: &str = "This is a slightly less important email";

    fn get_args() -> Vec<String> {
        vec![
            format!("{SMTP_SERVER_ARG}={TEST_SMTP_SERVER}"),
            format!("{SMTP_PORT_ARG}={TEST_SMTP_PORT}"),
            format!("{EMAIL_SENDER_NAME_ARG}={TEST_EMAIL_SENDER_NAME}"),
            format!("{EMAIL_SENDER_ADDRESS_ARG}={TEST_EMAIL_SENDER_ADDRESS}"),
            format!("{REPLY_TO_ARG}={TEST_REPLY_TO}"),
        ]
    }

    // region send_email
    #[async_test]
    #[ignore]
    async fn should_send_email() {
        let args = get_args();
        with_env_args(args, || {
            block_on(send_email(TEST_RECIPIENTS, TEST_SUBJECT, TEST_TEXT_BODY))
        })
        .unwrap();
    }
    // endregion

    // region create_message
    #[test]
    fn should_create_message() {
        let sender_name = "Sender";
        let sender_address = "sender@address.com";
        let sender_name_arg = format!("{EMAIL_SENDER_NAME_ARG}={sender_name}");
        let sender_address_arg = format!("{EMAIL_SENDER_ADDRESS_ARG}={sender_address}");
        let args = vec![sender_name_arg, sender_address_arg];

        let function = || create_message(TEST_RECIPIENTS, TEST_SUBJECT, TEST_TEXT_BODY);
        let result = with_env_args(args, function);

        assert!(result.is_ok());
        let result = result.unwrap();
        match result.clone().text_body.unwrap().contents {
            BodyPart::Text(text) => assert_eq!(TEST_TEXT_BODY, text),
            BodyPart::Binary(_) => panic!("Unexpected binary part"),
            BodyPart::Multipart(_) => panic!("Unexpected multipart part"),
        };
    }

    #[parameterized(
        args = {
            vec![format!("{EMAIL_SENDER_NAME_ARG}={TEST_EMAIL_SENDER_NAME}")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}={TEST_EMAIL_SENDER_ADDRESS}")],
            vec![],
        },
        expected_error = {
            MissingEmailSenderAddress,
            MissingEmailSenderName,
            MissingEmailSenderName,
        }
    )]
    fn should_fail_to_create_message(args: Vec<String>, expected_error: Error) {
        let function = || create_message(TEST_RECIPIENTS, TEST_SUBJECT, TEST_TEXT_BODY);
        let result = with_env_args(args, function);

        let error = result.unwrap_err();
        assert_eq!(expected_error, error);
    }
    // endregion

    // region Retrieve args
    #[parameterized(
        args = {
            vec![format!("{SMTP_SERVER_ARG}={TEST_SMTP_SERVER}")],
            vec![format!("{SMTP_PORT_ARG}={TEST_SMTP_PORT}")],
        },
        function = {
            &retrieve_smtp_server,
            & || retrieve_smtp_port().to_string(),
        },
        expected_result = {
            TEST_SMTP_SERVER.to_owned(),
            TEST_SMTP_PORT.to_string(),
        }
    )]
    fn should_retrieve_optional_arg(
        args: Vec<String>,
        function: &dyn Fn() -> String,
        expected_result: String,
    ) {
        let result = with_env_args(args, function);

        assert_eq!(expected_result, result);
    }

    #[parameterized(
        args = {
            vec![],
            vec![],
        },
        function = {
            &retrieve_smtp_server,
            & || retrieve_smtp_port().to_string(),
        },
        expected_result = {
            DEFAULT_SMTP_SERVER.to_owned(),
            DEFAULT_SMTP_PORT.to_string(),
        }
    )]
    fn should_retrieve_default_value_for_optional_arg(
        args: Vec<String>,
        function: &dyn Fn() -> String,
        expected_result: String,
    ) {
        let result = with_env_args(args, function);

        assert_eq!(expected_result, result);
    }

    #[parameterized(
        args = {
            vec![format!("{EMAIL_SENDER_NAME_ARG}={TEST_EMAIL_SENDER_NAME}")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}={TEST_EMAIL_SENDER_ADDRESS}")],
        },
        function = {
            &retrieve_email_sender_name,
            &retrieve_email_sender_address,
        },
        expected_result = {
            TEST_EMAIL_SENDER_NAME.to_owned(),
            TEST_EMAIL_SENDER_ADDRESS.to_owned(),
        }
    )]
    fn should_retrieve_expected_arg(
        args: Vec<String>,
        function: &dyn Fn() -> Result<String>,
        expected_result: String,
    ) {
        let result = with_env_args(args, function).unwrap();

        assert_eq!(expected_result, result);
    }

    #[parameterized(
        args = {
            vec![format!("{EMAIL_SENDER_NAME_ARG}={TEST_EMAIL_SENDER_NAME}")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}={TEST_EMAIL_SENDER_ADDRESS}")],
        },
        function = {
            &retrieve_email_sender_name,
            &retrieve_email_sender_address,
        },
        expected_result = {
            TEST_EMAIL_SENDER_NAME.to_owned(),
            TEST_EMAIL_SENDER_ADDRESS.to_owned(),
        }
    )]
    fn should_fail_to_retrieve_arg(
        args: Vec<String>,
        function: &dyn Fn() -> Result<String>,
        expected_result: String,
    ) {
        let result = with_env_args(args, function).unwrap();

        assert_eq!(expected_result, result);
    }
    // endregion
}
