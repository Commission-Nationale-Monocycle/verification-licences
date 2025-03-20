use std::fmt::{Debug, Display, Formatter};
use wasm_bindgen::JsValue;
use web_sys::{Element, Node};

pub const DEFAULT_ERROR_MESSAGE: &str = "Une erreur s'est produite. Veuillez réessayer";
pub const DEFAULT_SERVER_ERROR_MESSAGE: &str =
    "Le serveur a rencontré une erreur lors du traitement. Veuillez réessayer.";

pub struct Error {
    msg: String,
    technical_msg: String,
    parent: Option<Box<Error>>,
}

impl Error {
    pub fn new(msg: &str, technical_msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
            technical_msg: technical_msg.to_owned(),
            parent: None,
        }
    }

    pub fn from_parent(msg: &str, parent: Error) -> Self {
        Self {
            msg: msg.to_owned(),
            technical_msg: msg.to_owned(),
            parent: Some(Box::from(parent)),
        }
    }

    pub fn from_parent_with_default_message(parent: Error) -> Self {
        Self {
            msg: DEFAULT_ERROR_MESSAGE.to_owned(),
            technical_msg: DEFAULT_ERROR_MESSAGE.to_owned(),
            parent: Some(Box::from(parent)),
        }
    }

    pub fn from_server_status_error(status: u16) -> Self {
        Self {
            msg: DEFAULT_SERVER_ERROR_MESSAGE.to_owned(),
            technical_msg: format!("Server status: {status}"),
            parent: None,
        }
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            msg: DEFAULT_ERROR_MESSAGE.to_owned(),
            technical_msg: DEFAULT_ERROR_MESSAGE.to_owned(),
            parent: None,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.parent {
            None => {
                write!(f, "{}", self.technical_msg)
            }
            Some(parent) => {
                write!(f, "{}: caused by:\n{:?}", self.technical_msg, parent)
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::new(
            DEFAULT_ERROR_MESSAGE,
            &value
                .as_string()
                .unwrap_or("Unknown error has happened".to_owned()),
        )
    }
}

impl From<Element> for Error {
    fn from(element: Element) -> Self {
        let text = format!("A cast has failed for element: {element:?}");
        Self::new(DEFAULT_ERROR_MESSAGE, &text)
    }
}

impl From<Node> for Error {
    fn from(node: Node) -> Self {
        let text = format!("A cast has failed for node: {node:?}");
        Self::new(DEFAULT_ERROR_MESSAGE, &text)
    }
}
