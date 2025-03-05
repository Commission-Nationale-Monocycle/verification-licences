use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn create_element(
    document: &Document,
    name: &str,
    parent: Option<&Element>,
    inner_html: Option<&str>,
) -> Element {
    let new_element = document
        .create_element(name)
        .expect("can't create elements");

    if let Some(inner_html) = inner_html {
        new_element.set_inner_html(inner_html);
    }

    if let Some(parent) = parent {
        parent
            .append_child(&new_element)
            .expect("can't append child");
    }

    new_element
}

pub fn create_element_with_class(
    document: &Document,
    name: &str,
    parent: Option<&Element>,
    inner_html: Option<&str>,
    class: &str,
) -> Element {
    let new_element = create_element(document, name, parent, inner_html);
    new_element.set_class_name(class);
    new_element
}

pub fn create_element_with_classes(
    document: &Document,
    name: &str,
    parent: Option<&Element>,
    inner_html: Option<&str>,
    classes: &[&str],
) -> Element {
    let new_element = create_element(document, name, parent, inner_html);
    new_element.set_class_name(&classes.join(" "));
    new_element
}

pub fn get_window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn get_document() -> Document {
    let window = get_window();
    window.document().expect("should have a document on window")
}

pub fn get_element_by_id(document: &Document, id: &str) -> Element {
    document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("`{id}` element does not exist"))
}

pub fn get_element_by_id_dyn<T: JsCast>(document: &Document, id: &str) -> T {
    get_element_by_id(document, id).dyn_into().unwrap()
}

pub fn get_value_from_input(document: &Document, id: &str) -> String {
    get_element_by_id(document, id)
        .get_attribute("value")
        .unwrap_or_else(|| panic!("`{id}` input does not contain text"))
}

pub fn append_child(container: &Element, child: &Element) {
    container.append_child(child).expect("can't append child");
}

pub fn clear_element(element: &Element) {
    element.set_inner_html("");
}

pub fn set_attribute(element: &Element, name: &str, value: &str) {
    element
        .set_attribute(name, value)
        .expect("can't set attribute");
}

pub fn remove_attribute(element: &Element, name: &str) {
    element
        .remove_attribute(name)
        .expect("can't remove attribute");
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn should_get_window() {
        get_window();
    }
}
