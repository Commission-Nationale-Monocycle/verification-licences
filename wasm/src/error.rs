use std::fmt::{Debug, Display, Formatter};
use wasm_bindgen::JsValue;
use web_sys::{Element, Node};

pub struct Error {
    msg: String,
    parent: Option<Box<Error>>,
}

impl Error {
    pub fn new(msg: String) -> Self {
        Self { msg, parent: None }
    }

    pub fn from_parent(msg: String, parent: Error) -> Self {
        Self {
            msg,
            parent: Some(Box::from(parent)),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: caused by:\n{:?}", self.msg, self.parent)
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
            value
                .as_string()
                .unwrap_or("Unknown error has happened".to_owned()),
        )
    }
}

impl From<Element> for Error {
    fn from(element: Element) -> Self {
        let text = format!("A cast has failed for element: {element:?}");
        Self::new(text)
    }
}

impl From<Node> for Error {
    fn from(node: Node) -> Self {
        let text = format!("A cast has failed for node: {node:?}");
        Self::new(text)
    }
}
