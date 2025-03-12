#[cfg(not(test))]
use crate::template::get_template;
#[cfg(not(test))]
use crate::utils::{append_child, get_body};
use crate::utils::{get_element_by_id, query_selector_single_element};
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(not(test))]
use web_sys::Element;
use web_sys::{Document, Node};

#[wasm_bindgen]
pub enum AlertLevel {
    Info = 0,
    Error = 1,
}

#[cfg(not(test))]
fn get_alert_template(document: &Document, level: &AlertLevel) -> Element {
    match level {
        AlertLevel::Info => get_template(document, "alert-info"),
        AlertLevel::Error => get_template(document, "alert-error"),
    }
}

#[cfg(not(test))]
#[wasm_bindgen]
pub fn create_alert(document: &Document, text: &str, level: AlertLevel) {
    document
        .get_element_by_id("alert")
        .as_ref()
        .map(Element::remove);

    let alert = get_alert_template(document, &level);
    let content_container = query_selector_single_element(document, &alert, ".alert-content");
    content_container.set_inner_html(text);

    append_child(&get_body(), &alert);

    Dismiss::new(
        &alert,
        &query_selector_single_element(document, &alert, "#close-alert"),
    );
}

#[cfg(test)]
pub fn create_alert(_document: &Document, _text: &str, _level: AlertLevel) {
    // Nothing to do for tests
}

#[wasm_bindgen]
pub fn hide_alert(document: &Document) {
    let element = get_element_by_id(document, "alert");
    let alert = Dismiss::new(
        &element,
        &query_selector_single_element(document, &element, "#close-alert"),
    );
    alert.hide();
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
