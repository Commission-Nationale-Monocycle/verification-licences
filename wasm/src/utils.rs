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

// region Get elements
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
// endregion

// region Create elements
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
// endregion

// region Manipulate existing elements
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
// endregion

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    fn get_new_document() -> Document {
        Document::new().unwrap()
    }

    // region Get elements
    #[wasm_bindgen_test]
    fn should_get_window() {
        get_window();
    }

    #[wasm_bindgen_test]
    fn should_get_document() {
        get_document();
    }

    #[wasm_bindgen_test]
    fn should_get_element_by_id() {
        let id = "id";

        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);

        document.get_root_node().append_child(&element).unwrap();

        get_element_by_id(&document, id);
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "element does not exist")]
    fn should_not_get_element_by_id() {
        let id = "id";

        let document = get_new_document();
        get_element_by_id(&document, id);
    }

    #[wasm_bindgen_test]
    fn should_get_element_by_id_dyn() {
        let id = "id";
        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);

        document.get_root_node().append_child(&element).unwrap();
        get_element_by_id_dyn::<Element>(&document, id);
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "element does not exist")]
    fn should_not_get_element_by_id_dyn() {
        let id = "id";

        let document = get_new_document();
        get_element_by_id_dyn::<Element>(&document, id);
    }

    #[wasm_bindgen_test]
    fn should_get_value_from_input() {
        let id = "id";
        let value = "value";

        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);
        element.set_attribute("value", value).unwrap();

        document.get_root_node().append_child(&element).unwrap();

        assert_eq!(value, get_value_from_input(&document, id));
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "element does not exist")]
    fn should_not_get_value_when_element_does_not_exist() {
        let id = "id";

        let document = get_new_document();
        get_value_from_input(&document, id);
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "input does not contain text")]
    fn should_not_get_value_when_element_does_not_not_contain_text() {
        let id = "id";

        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);

        document.get_root_node().append_child(&element).unwrap();

        get_value_from_input(&document, id);
    }
    // endregion

    // region Create elements
    #[wasm_bindgen_test]
    fn should_create_element() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = Some("some text");

        let new_element = create_element(&document, name, Some(&parent), inner_html);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html.unwrap(), new_element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_create_element_without_parent_or_text() {
        let document = Document::new().unwrap();
        let name = "p";

        let new_element = create_element(&document, name, None, None);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(None, new_element.parent_element());
        assert_eq!("", new_element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_create_element_with_class() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = Some("some text");
        let class = "class";

        let new_element =
            create_element_with_class(&document, name, Some(&parent), inner_html, class);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html.unwrap(), new_element.inner_html());
        assert_eq!(class, new_element.class_name());
    }

    #[wasm_bindgen_test]
    fn should_create_element_with_classes() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = Some("some text");
        let classes = ["class1", "class2"];

        let new_element =
            create_element_with_classes(&document, name, Some(&parent), inner_html, &classes);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html.unwrap(), new_element.inner_html());
        assert_eq!(classes.join(" "), new_element.class_name());
    }
    // endregion

    // region Manipulate existing elements
    #[wasm_bindgen_test]
    fn should_append_chile() {
        let document = Document::new().unwrap();
        let child = document.create_element("p").unwrap();
        let container = document.create_element("p").unwrap();

        append_child(&container, &child);
    }

    #[wasm_bindgen_test]
    fn should_clear_element() {
        let document = Document::new().unwrap();
        let element = document.create_element("p").unwrap();
        let value = "value";
        element.set_inner_html(value);

        assert_eq!(value, element.inner_html());

        clear_element(&element);

        assert_eq!("", element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_set_attribute() {
        let document = Document::new().unwrap();
        let element = document.create_element("p").unwrap();
        let key = "key";
        let value = "value";

        assert_eq!(None, element.get_attribute(key));
        set_attribute(&element, key, value);
        assert_eq!(value, element.get_attribute(key).unwrap());
    }

    #[wasm_bindgen_test]
    fn should_remove_attribute() {
        let document = Document::new().unwrap();
        let element = document.create_element("p").unwrap();
        let key = "key";
        let value = "value";

        element
            .set_attribute(key, value)
            .expect("can't set attribute");
        assert_eq!(value, element.get_attribute(key).unwrap());
        remove_attribute(&element, key);
        assert_eq!(None, element.get_attribute(key));
    }
    // endregion
}
