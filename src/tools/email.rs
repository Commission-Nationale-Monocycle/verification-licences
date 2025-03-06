use crate::tools::email::Error::{
    CantConnectToSmtpServer, CantSendMessage, MissingEmailSenderAddress, MissingEmailSenderName,
    MissingSmtpLogin, MissingSmtpPassword,
};
use crate::tools::env_args::retrieve_arg_value;
use crate::tools::log_message_and_return;
use mail_send::SmtpClientBuilder;
use mail_send::mail_builder::MessageBuilder;

type Result<T, E = Error> = std::result::Result<T, E>;

const EMAIL_SENDER_NAME_ARG: &'static str = "--email_sender_name";
const EMAIL_SENDER_ADDRESS_ARG: &'static str = "--email_address";

pub async fn send_email(
    recipients: &[(&str, &str)],
    subject: &str,
    html_body: &str,
    text_body: &str,
) -> Result<()> {
    let message = create_message(recipients, subject, html_body, text_body)?;

    const GMAIL_SMTP_SERVER: &'static str = "smtp.gmail.com";
    const GMAIL_SMTP_PORT: u16 = 587;
    let gmail_smtp_login = retrieve_arg_value(&["--email_sender_name"]).ok_or(MissingSmtpLogin)?;
    let gmail_smtp_password =
        retrieve_arg_value(&["--email_sender_name"]).ok_or(MissingSmtpPassword)?;
    let smtp_client = SmtpClientBuilder::new(GMAIL_SMTP_SERVER, GMAIL_SMTP_PORT)
        .implicit_tls(false)
        .credentials((gmail_smtp_login.as_str(), gmail_smtp_password.as_str()))
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
    let sender_name = retrieve_arg_value(&[EMAIL_SENDER_NAME_ARG]).ok_or(MissingEmailSenderName)?;
    let sender_address =
        retrieve_arg_value(&[EMAIL_SENDER_ADDRESS_ARG]).ok_or(MissingEmailSenderAddress)?;

    Ok(MessageBuilder::new()
        .from((sender_name, sender_address))
        .to(Vec::from(recipients))
        .subject(subject)
        .html_body(html_body)
        .text_body(text_body))
}

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
            vec![format!("{EMAIL_SENDER_NAME_ARG}=sender")],
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
}
