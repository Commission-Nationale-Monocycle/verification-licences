use crate::utils::get_element_by_id;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element};

#[allow(dead_code)]
fn get_toast_success(document: &Document) -> Element {
    get_element_by_id(document, "toast_success")
}

#[allow(dead_code)]
fn get_toast_error(document: &Document) -> Element {
    get_element_by_id(document, "toast_error")
}

#[derive(Serialize)]
struct ModalOptions {
    placement: String,
    backdrop_classes: String,
    backdrop: String,
    closeable: bool,
}

impl ModalOptions {
    pub fn new(
        placement: String,
        backdrop_classes: String,
        backdrop: String,
        closeable: bool,
    ) -> Self {
        Self {
            placement,
            backdrop_classes,
            backdrop,
            closeable,
        }
    }
}

fn create_toast_options() -> ModalOptions {
    ModalOptions::new(
        "center".to_owned(),
        "test".to_owned(),
        "dynamic".to_owned(),
        true,
    )
}

#[wasm_bindgen]
pub fn show_toast(document: &Document, toast_id: &str, text: &str) {
    let toast = get_element_by_id(document, toast_id);
    let modal = Modal::new(
        &toast,
        &serde_wasm_bindgen::to_value(&create_toast_options()).unwrap(),
    );
    let element = toast
        .get_elements_by_class_name("toast-text")
        .get_with_index(0)
        .unwrap();
    element.set_text_content(Some(text));
    modal.show();
}

#[wasm_bindgen]
pub fn hide_toast(document: &Document, toast_id: &str) {
    let toast = get_element_by_id(document, toast_id);
    let modal = Modal::new(
        &toast,
        &serde_wasm_bindgen::to_value(&create_toast_options()).unwrap(),
    );
    modal.hide();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = Flowbite)]
    type Modal;

    #[wasm_bindgen(constructor)]
    fn new(element: &Element, options: &JsValue) -> Modal;

    #[wasm_bindgen(method)]
    fn show(this: &Modal);

    #[wasm_bindgen(method)]
    fn hide(this: &Modal);
}
