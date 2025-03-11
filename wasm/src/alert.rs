use crate::utils::{append_child, get_body, get_element_by_id, query_selector_single_element};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, DocumentFragment, Element, HtmlTemplateElement, Node};

#[wasm_bindgen]
pub enum AlertLevel {
    Info = 0,
    Error = 1,
}

#[cfg(not(test))]
fn get_alert_template(document: &Document, level: &AlertLevel) -> HtmlTemplateElement {
    match level {
        AlertLevel::Info => get_element_by_id(document, "alert_info"),
        AlertLevel::Error => get_element_by_id(document, "alert_error"),
    }
    .dyn_into::<HtmlTemplateElement>()
    .expect("Couldn't retrieve Alert template")
}

#[cfg(not(test))]
#[wasm_bindgen]
pub fn create_alert(document: &Document, text: &str, level: AlertLevel) {
    document
        .get_element_by_id("alert")
        .as_ref()
        .map(Element::remove);

    let body = get_body();
    let template = get_alert_template(document, &level);
    let alert = template
        .content()
        .clone_node_with_deep(true)
        .unwrap_or_else(|error| panic!("Couldn't clone node: {error:?}"))
        .dyn_into::<DocumentFragment>()
        .unwrap();

    append_child(&body, &alert);

    let alert = get_element_by_id(document, "alert");
    let content_container = query_selector_single_element(document, &alert, ".alert-content");
    content_container.set_inner_html(text);

    Dismiss::new(
        &alert,
        &query_selector_single_element(document, &alert, "#close_alert"),
    );
}

#[cfg(test)]
pub fn show_alert(_document: &Document, _text: &str, _level: AlertLevel) {
    // Nothing to do for tests
}

#[wasm_bindgen]
pub fn hide_alert(document: &Document) {
    let element = get_element_by_id(document, "alert");
    let alert = Dismiss::new(
        &element,
        &query_selector_single_element(document, &element, "#close_alert"),
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
