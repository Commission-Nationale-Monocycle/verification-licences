use crate::Result;
#[cfg(not(test))]
use crate::template::get_template;
#[cfg(not(test))]
use crate::utils::{append_child, get_body};
use crate::utils::{get_document, get_element_by_id, query_selector_single_element};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Node;
#[cfg(not(test))]
use web_sys::{Document, Element};

#[wasm_bindgen]
pub enum AlertLevel {
    Info = 0,
    Error = 1,
}

#[cfg(not(test))]
fn get_alert_template(document: &Document, level: &AlertLevel) -> Result<Element> {
    match level {
        AlertLevel::Info => get_template(document, "alert-info"),
        AlertLevel::Error => get_template(document, "alert-error"),
    }
}

#[cfg(not(test))]
#[wasm_bindgen]
pub fn create_alert(text: &str, level: AlertLevel) {
    let document = unwrap_without_alert(get_document());
    document
        .get_element_by_id("alert")
        .as_ref()
        .map(Element::remove);

    let alert = unwrap_without_alert(get_alert_template(&document, &level));
    let content_container =
        unwrap_without_alert(query_selector_single_element(&alert, ".alert-content"));
    content_container.set_inner_html(text);

    let body = unwrap_without_alert(get_body());
    unwrap_without_alert(append_child(&body, &alert));

    let clickable_element =
        unwrap_without_alert(query_selector_single_element(&alert, "#close-alert"));
    Dismiss::new(&alert, &clickable_element);
}

#[cfg(test)]
pub fn create_alert(_text: &str, _level: AlertLevel) {
    // Nothing to do for tests
}

#[wasm_bindgen]
pub fn hide_alert() {
    let document = &unwrap_without_alert(get_document());
    let element = unwrap_without_alert(get_element_by_id(document, "alert"));
    let clickable_element =
        unwrap_without_alert(query_selector_single_element(&element, "#close-alert"));
    let alert = Dismiss::new(&element, &clickable_element);
    alert.hide();
}

/// Unwrap the result if it is some. Otherwise, create an alert for the user.
pub fn unwrap_or_alert<T>(result: Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            create_alert(&error.to_string(), AlertLevel::Error);
            log::error!("{:#?}", error);
            panic!("{error:#?}");
        }
    }
}

/// When it's impossible to create an alert, this method should be used instead of [unwrap_or_alert].
/// Otherwise, it may trigger an infinite loop.
pub fn unwrap_without_alert<T>(result: Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            log::error!("{:#?}", error);
            panic!("{error:#?}");
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = Flowbite)]
    type Dismiss;

    #[wasm_bindgen(constructor)]
    fn new(element: &Node, dismiss_on_click_element: &Node) -> Dismiss;

    #[wasm_bindgen(method)]
    fn hide(this: &Dismiss);
}
