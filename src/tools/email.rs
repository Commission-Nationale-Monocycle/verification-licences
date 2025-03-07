use crate::tools::email::Error::{
    CantConnectToSmtpServer, CantSendMessage, MissingEmailSenderAddress, MissingEmailSenderName,
    MissingSmtpLogin, MissingSmtpPassword,
};
use crate::tools::env_args::{retrieve_arg_value, retrieve_expected_arg_value};
use crate::tools::log_message_and_return;
use mail_send::SmtpClientBuilder;
use mail_send::mail_builder::MessageBuilder;

type Result<T, E = Error> = std::result::Result<T, E>;

const EMAIL_SENDER_NAME_ARG: &'static str = "--email-sender-name";
const EMAIL_SENDER_ADDRESS_ARG: &'static str = "--email-sender_address";
const REPLY_TO_ARG: &'static str = "--reply-to";
const SMTP_SERVER_ARG: &'static str = "--smtp-server";
const SMTP_PORT_ARG: &'static str = "--smtp-port";
const SMTP_LOGIN_ARG: &'static str = "--smtp-login";
const SMTP_PASSWORD_ARG: &'static str = "--smtp-password";
const DEFAULT_SMTP_SERVER: &'static str = "smtp.gmail.com";
const DEFAULT_SMTP_PORT: u16 = 25;

pub async fn send_email(
    recipients: &[(&str, &str)],
    subject: &str,
    html_body: &str,
    text_body: &str,
) -> Result<()> {
    let message = create_message(recipients, subject, html_body, text_body)?;
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
    recipients: &'a [(&str, &str)],
    subject: &'a str,
    html_body: &'a str,
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
        .html_body(html_body)
        .text_body(text_body))
}

// region Retrieve args
fn retrieve_smtp_server() -> String {
    retrieve_arg_value(SMTP_SERVER_ARG).unwrap_or(DEFAULT_SMTP_SERVER.to_owned())
}
fn retrieve_smtp_port() -> u16 {
    retrieve_arg_value(SMTP_PORT_ARG)
        .map(|port| port.parse::<u16>().ok())
        .flatten()
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

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingEmailSenderName,
    MissingEmailSenderAddress,
    MissingSmtpLogin,
    MissingSmtpPassword,
    CantConnectToSmtpServer,
    CantSendMessage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::env_args::with_env_args;
    use mail_send::mail_builder::mime::BodyPart;
    use parameterized::{ide, parameterized};

    ide!();

    // region create_message
    #[test]
    fn should_create_message() {
        let sender_name = "Sender";
        let sender_address = "sender@address.com";
        let sender_name_arg = format!("{EMAIL_SENDER_NAME_ARG}={sender_name}");
        let sender_address_arg = format!("{EMAIL_SENDER_ADDRESS_ARG}={sender_address}");
        let args = vec![sender_name_arg, sender_address_arg];

        let recipient = ("Recipient", "recipient@address.com");
        let recipients = &[recipient];
        let subject = "This is an important mail!";
        let html_body = "<h1>Important!</h1>";
        let text_body = "Important!";

        let function = || create_message(recipients, subject, html_body, text_body);
        let result = with_env_args(args, function);

        assert!(result.is_ok());
        let result = result.unwrap();
        match result.clone().text_body.unwrap().contents {
            BodyPart::Text(text) => assert_eq!(text_body, text),
            BodyPart::Binary(_) => panic!("Unexpected binary part"),
            BodyPart::Multipart(_) => panic!("Unexpected multipart part"),
        };
    }

    #[parameterized(
        args = {
            vec![format!("{EMAIL_SENDER_NAME_ARG}=Sender")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}=sender@address.com")],
            vec![],
        },
        expected_error = {
            MissingEmailSenderAddress,
            MissingEmailSenderName,
            MissingEmailSenderName,
        }
    )]
    fn should_fail_to_create_message(args: Vec<String>, expected_error: Error) {
        let recipient = ("Recipient", "recipient@address.com");
        let recipients = &[recipient];
        let subject = "This is an important mail!";
        let html_body = "<h1>Important!</h1>";
        let text_body = "Important!";

        let function = || create_message(recipients, subject, html_body, text_body);
        let result = with_env_args(args, function);

        let error = result.unwrap_err();
        assert_eq!(expected_error, error);
    }
    // endregion

    // region Retrieve args
    #[parameterized(
        args = {
            vec![format!("{SMTP_LOGIN_ARG}=login")],
            vec![format!("{SMTP_PASSWORD_ARG}=password")],
            vec![format!("{EMAIL_SENDER_NAME_ARG}=Sender")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}=sender@address.com")],
            vec![format!("{SMTP_LOGIN_ARG}=login"), format!("{SMTP_PASSWORD_ARG}=password")],
        },
        function = {
            &retrieve_smtp_login,
            &retrieve_smtp_password,
            &retrieve_email_sender_name,
            &retrieve_email_sender_address,
            &retrieve_smtp_login
        },
        expected_result = {
            "login".to_owned(),
            "password".to_owned(),
            "Sender".to_owned(),
            "sender@address.com".to_owned(),
            "login".to_owned(),
        }
    )]
    fn should_retrieve_arg(
        args: Vec<String>,
        function: &dyn Fn() -> Result<String>,
        expected_result: String,
    ) {
        let result = with_env_args(args, function).unwrap();

        assert_eq!(expected_result, result);
    }

    #[parameterized(
        args = {
            vec![format!("{SMTP_LOGIN_ARG}=login")],
            vec![format!("{SMTP_PASSWORD_ARG}=password")],
            vec![format!("{EMAIL_SENDER_NAME_ARG}=Sender")],
            vec![format!("{EMAIL_SENDER_ADDRESS_ARG}=sender@address.com")],
            vec![format!("{SMTP_LOGIN_ARG}=login"), format!("{SMTP_PASSWORD_ARG}=password")],
        },
        function = {
            &retrieve_smtp_login,
            &retrieve_smtp_password,
            &retrieve_email_sender_name,
            &retrieve_email_sender_address,
            &retrieve_smtp_login
        },
        expected_result = {
            "login".to_owned(),
            "password".to_owned(),
            "Sender".to_owned(),
            "sender@address.com".to_owned(),
            "login".to_owned(),
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
