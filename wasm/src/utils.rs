use wasm_bindgen::{JsCast, UnwrapThrowExt};
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
pub struct ElementConfig<'a> {
    classes: Option<&'a [&'a str]>,
    attributes: Option<&'a [(&'a str, &'a str)]>,
    children: Option<&'a [&'a Element]>,
}

impl<'a> ElementConfig<'a> {
    pub fn new(
        classes: Option<&'a [&'a str]>,
        attributes: Option<&'a [(&'a str, &'a str)]>,
        children: Option<&'a [&'a Element]>,
    ) -> Self {
        Self {
            classes,
            attributes,
            children,
        }
    }

    pub fn classes(&self) -> Option<&'a [&'a str]> {
        self.classes
    }

    pub fn attributes(&self) -> Option<&'a [(&'a str, &'a str)]> {
        self.attributes
    }

    pub fn children(&self) -> Option<&'a [&'a Element]> {
        self.children
    }
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
    create_element_with_options(
        document,
        name,
        parent,
        inner_html,
        &ElementConfig::new(Some(&[class]), None, None),
    )
}

pub fn create_element_with_classes(
    document: &Document,
    name: &str,
    parent: Option<&Element>,
    inner_html: Option<&str>,
    classes: &[&str],
) -> Element {
    create_element_with_options(
        document,
        name,
        parent,
        inner_html,
        &ElementConfig::new(Some(classes), None, None),
    )
}

pub fn create_element_with_options(
    document: &Document,
    name: &str,
    parent: Option<&Element>,
    inner_html: Option<&str>,
    config: &ElementConfig,
) -> Element {
    let new_element = create_element(document, name, parent, inner_html);
    if let Some(classes) = config.classes() {
        new_element.set_class_name(&classes.join(" "));
    }
    if let Some(attributes) = config.attributes() {
        attributes.iter().for_each(|(name, value)| {
            new_element
                .set_attribute(name, value)
                .expect_throw("can't set attribute");
        });
    }
    if let Some(children) = config.children() {
        children.iter().for_each(|child| {
            append_child(&new_element, child);
        });
    }

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

        let new_element = create_element(&document, name, Some(&parent), Some(inner_html));

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html, new_element.inner_html());
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
        let inner_html = "some text";
        let class = "class";

        let new_element =
            create_element_with_class(&document, name, Some(&parent), Some(inner_html), class);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html, new_element.inner_html());
        assert_eq!(class, new_element.class_name());
    }

    #[wasm_bindgen_test]
    fn should_create_element_with_classes() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = "some text";
        let classes = ["class1", "class2"];

        let new_element =
            create_element_with_classes(&document, name, Some(&parent), Some(inner_html), &classes);

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html, new_element.inner_html());
        assert_eq!(classes.join(" "), new_element.class_name());
    }

    #[wasm_bindgen_test]
    fn should_create_element_with_options() {
        let document = Document::new().unwrap();
        let name = "p";
        let parent = document.create_element("p").unwrap();
        let inner_html = "some text";
        let classes = ["class1", "class2"];
        let attributes = [("name1", "value1"), ("name2", "value2")];
        let child1 = create_element(&document, "p", None, None);
        let child2 = create_element(&document, "div", None, None);

        let new_element = create_element_with_options(
            &document,
            name,
            Some(&parent),
            Some(inner_html),
            &ElementConfig::new(Some(&classes), Some(&attributes), Some(&[&child1, &child2])),
        );

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert!(new_element.inner_html().starts_with(inner_html));
        assert_eq!(classes.join(" "), new_element.class_name());
        assert_eq!(
            attributes[0].1,
            new_element
                .attributes()
                .get_with_name(attributes[0].0)
                .unwrap()
                .value()
        );
        assert_eq!(
            attributes[1].1,
            new_element
                .attributes()
                .get_with_name(attributes[1].0)
                .unwrap()
                .value()
        );
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
