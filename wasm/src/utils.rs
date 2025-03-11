use crate::alert::{AlertLevel, create_alert};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, Location, Node};

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

pub fn get_body() -> HtmlElement {
    let document = get_document();
    document.body().expect("should have a document on window")
}

pub fn get_element_by_id(document: &Document, id: &str) -> Element {
    document.get_element_by_id(id).unwrap_or_else(|| {
        create_alert(
            document,
            "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
            AlertLevel::Error,
        );

        panic!("`{id}` element does not exist");
    })
}

pub fn get_element_by_id_dyn<T: JsCast>(document: &Document, id: &str) -> T {
    get_element_by_id(document, id)
        .dyn_into()
        .unwrap_or_else(|error| {
            create_alert(
                document,
                "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                AlertLevel::Error,
            );
            panic!("Can't cast element: {error:?}");
        })
}

pub fn query_selector_single_element(
    document: &Document,
    element: &Element,
    selector: &str,
) -> Element {
    element
        .query_selector(selector)
        .unwrap_or_else(|error| {
            create_alert(
                document,
                "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                AlertLevel::Error,
            );
            panic!("There should be a single element matching query: {error:?}.")
        })
        .unwrap_or_else(|| {
            create_alert(
                document,
                "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                AlertLevel::Error,
            );
            panic!("There should be a single element matching query.")
        })
}

pub fn get_value_from_input(document: &Document, id: &str) -> String {
    get_element_by_id(document, id)
        .get_attribute("value")
        .unwrap_or_else(|| {
            create_alert(
                document,
                "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                AlertLevel::Error,
            );
            panic!("`{id}` input does not contain text");
        })
}
// endregion

// region Create elements
#[derive(Default)]
pub struct ElementBuilder<'a> {
    parent: Option<&'a Element>,
    inner_html: Option<&'a str>,
}

impl<'a> ElementBuilder<'a> {
    pub fn parent(mut self, parent: &'a Element) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn inner_html(mut self, inner_html: &'a str) -> Self {
        self.inner_html = Some(inner_html);
        self
    }

    pub fn build(&self, document: &Document, name: &str) -> Element {
        let new_element = document
            .create_element(name)
            .expect("can't create elements");

        if let Some(parent) = self.parent {
            parent
                .append_child(&new_element)
                .expect("can't append child");
        }
        if let Some(inner_html) = self.inner_html {
            new_element.set_inner_html(inner_html);
        }

        new_element
    }
}

pub fn create_element(document: &Document, name: &str) -> Element {
    ElementBuilder::default().build(document, name)
}
// endregion

// region Manipulate existing elements
pub fn append_child(container: &Element, child: &Node) {
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

pub fn add_class(element: &Element, class_name: &str) {
    let already_applied_classes = element.class_name();
    if already_applied_classes.is_empty() {
        element.set_class_name(class_name);
    } else {
        let new_set_of_classes = format!("{} {}", already_applied_classes, class_name);
        element.set_class_name(&new_set_of_classes);
    }
}

pub fn remove_class(element: &Element, class_name: &str) {
    let classes = element.class_name();
    if !classes.contains(class_name) {
        return;
    }
    let new_set_of_classes = classes
        .split(" ")
        .filter(|applied_class| *applied_class != class_name)
        .collect::<Vec<_>>()
        .join(" ");

    element.set_class_name(&new_set_of_classes);
}
// endregion

// region Location
pub fn get_location() -> Location {
    get_window().location()
}

pub fn get_origin() -> String {
    get_location().origin().unwrap_or_else(|error| {
        create_alert(
            &get_document(),
            "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
            AlertLevel::Error,
        );
        panic!("Couldn't get origin: {error:?}")
    })
}

pub fn get_pathname() -> String {
    get_location().pathname().unwrap_or_else(|error| {
        create_alert(
            &get_document(),
            "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
            AlertLevel::Error,
        );
        panic!("Couldn't get origin: {error:?}")
    })
}

pub fn get_url_without_query() -> String {
    format!("{}{}", get_origin(), get_pathname())
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
        let inner_html = "some text";

        let new_element = ElementBuilder::default()
            .parent(&parent)
            .inner_html(inner_html)
            .build(&document, name);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html, new_element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_create_element_without_parent_or_text() {
        let document = Document::new().unwrap();
        let name = "p";

        let new_element = create_element(&document, name);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(None, new_element.parent_element());
        assert_eq!("", new_element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_create_element_with_options() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = "some text";

        let new_element = ElementBuilder::default()
            .parent(&parent)
            .inner_html(inner_html)
            .build(&document, name);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert!(new_element.inner_html().starts_with(inner_html));
    }
    // endregion

    // region Manipulate existing elements
    #[wasm_bindgen_test]
    fn should_append_child() {
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

    #[wasm_bindgen_test]
    fn should_add_new_class_when_no_class() {
        let document = Document::new().unwrap();
        let element = document.create_element("p").unwrap();
        let new_class = "class-name";

        add_class(&element, new_class);

        assert_eq!(new_class.to_owned(), element.class_name().trim());
    }

    #[wasm_bindgen_test]
    fn should_add_new_class_when_already_classes() {
        let document = Document::new().unwrap();
        let element = document.create_element("p").unwrap();
        let previous_classes = "older-class old-class";
        element.set_class_name(previous_classes);

        let new_class = "class-name";

        add_class(&element, new_class);

        assert!(
            element.class_name().contains(previous_classes),
            "The element does not have the previous class any more."
        );
        assert!(
            element.class_name().contains(new_class),
            "The element didn't get the new class."
        );
    }
    // endregion
}
