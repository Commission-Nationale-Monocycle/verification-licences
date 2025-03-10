use crate::utils::get_element_by_id;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element};

#[wasm_bindgen]
pub enum ToastLevel {
    INFO = 0,
    ERROR = 1,
}

fn get_toast_level_icon(toast_level: &ToastLevel) -> &'static str {
    match toast_level {
        ToastLevel::INFO => {
            r##"<div class="rounded-lg text-green-500 bg-green-100 dark:bg-green-800 dark:text-green-200"><svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20"><path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z"/></svg><span class="sr-only">Check icon</span></div>"##
        }
        ToastLevel::ERROR => {
            r##"<div class="rounded-lg text-red-500 bg-red-100 dark:bg-red-800 dark:text-red-200"><svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20"><path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/></svg><span class="sr-only">Error icon</span></div>"##
        }
    }
}

#[cfg(not(test))]
#[wasm_bindgen]
pub fn show_toast(document: &Document, text: &str, level: ToastLevel) {
    let toast = get_element_by_id(document, "toast");
    let modal = Modal::new(&toast);
    let icon = get_toast_level_icon(&level);
    let toast_icon_container = get_element_by_id(document, "toast_icon");
    toast_icon_container.set_inner_html(icon);
    let text_container = get_element_by_id(document, "toast_text");
    text_container.set_text_content(Some(text));
    modal.show();
}

#[cfg(test)]
pub fn show_toast(_document: &Document, _text: &str, _level: ToastLevel) {
    // Nothing to do for tests
}

#[wasm_bindgen]
pub fn hide_toast(document: &Document) {
    let toast = get_element_by_id(document, "toast");
    let modal = Modal::new(&toast);
    modal.hide();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = Flowbite)]
    type Modal;

    #[wasm_bindgen(constructor)]
    fn new(element: &Element) -> Modal;

    #[wasm_bindgen(method)]
    fn show(this: &Modal);

    #[wasm_bindgen(method)]
    fn hide(this: &Modal);
}
