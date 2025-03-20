use crate::Result;
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, Location, Node, Window};

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
pub fn get_window() -> Result<Window> {
    web_sys::window().ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "no global `window` exists"))
}

pub fn get_document() -> Result<Document> {
    let window = get_window()?;
    window
        .document()
        .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "should have a document on window"))
}

pub fn get_body() -> Result<HtmlElement> {
    let document = get_document()?;
    document
        .body()
        .ok_or_else(|| Error::new(DEFAULT_ERROR_MESSAGE, "should have a document on window"))
}

pub fn get_element_by_id(document: &Document, id: &str) -> Result<Element> {
    document.get_element_by_id(id).ok_or_else(|| {
        Error::new(
            DEFAULT_ERROR_MESSAGE,
            &format!("`{id}` element does not exist"),
        )
    })
}

pub fn get_element_by_id_dyn<T: JsCast>(document: &Document, id: &str) -> Result<T> {
    get_element_by_id(document, id)?
        .dyn_into()
        .map_err(|error| {
            Error::new(
                DEFAULT_ERROR_MESSAGE,
                &format!("Can't cast element: {error:?}"),
            )
        })
}

pub fn query_selector_single_element(element: &Element, selector: &str) -> Result<Element> {
    element.query_selector(selector)?.ok_or_else(|| {
        Error::new(
            DEFAULT_ERROR_MESSAGE,
            &format!("There should be a single element matching query [selector: {selector}]."),
        )
    })
}

pub fn query_selector_all(element: &Element, selector: &str) -> Result<Vec<Element>> {
    let node_list = element.query_selector_all(selector)?;
    let mut elements = vec![];
    for i in 0..node_list.length() {
        elements.push(node_list.get(i).unwrap().dyn_into()?);
    }
    Ok(elements)
}

pub fn get_value_from_element(element: &HtmlInputElement) -> String {
    element.value()
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

    pub fn build(&self, document: &Document, name: &str) -> Result<Element> {
        let new_element = document.create_element(name)?;

        if let Some(parent) = self.parent {
            parent.append_child(&new_element)?;
        }
        if let Some(inner_html) = self.inner_html {
            new_element.set_inner_html(inner_html);
        }

        Ok(new_element)
    }
}

pub fn create_element(document: &Document, name: &str) -> Result<Element> {
    ElementBuilder::default().build(document, name)
}
// endregion

// region Manipulate existing elements
pub fn append_child(container: &Element, child: &Node) -> Result<Node> {
    Ok(container.append_child(child)?)
}

pub fn clear_element(element: &Element) {
    element.set_inner_html("");
}

pub fn set_attribute(element: &Element, name: &str, value: &str) -> Result<()> {
    Ok(element.set_attribute(name, value)?)
}

pub fn remove_attribute(element: &Element, name: &str) -> Result<()> {
    Ok(element.remove_attribute(name)?)
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
pub fn get_location() -> Result<Location> {
    let window = get_window()?;
    Ok(window.location())
}

pub fn get_origin() -> Result<String> {
    let location = get_location()?;
    Ok(location.origin()?)
}

pub fn get_pathname() -> Result<String> {
    let location = get_location()?;
    Ok(location.pathname()?)
}

pub fn get_url_without_query() -> Result<String> {
    Ok(format!("{}{}", get_origin()?, get_pathname()?))
}
// endregion

#[allow(dead_code)]
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
        get_window().unwrap();
    }

    #[wasm_bindgen_test]
    fn should_get_document() {
        get_document().unwrap();
    }

    #[wasm_bindgen_test]
    fn should_get_element_by_id() {
        let id = "id";

        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);

        document.get_root_node().append_child(&element).unwrap();

        get_element_by_id(&document, id).unwrap();
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "element does not exist")]
    fn should_not_get_element_by_id() {
        let id = "id";

        let document = get_new_document();
        get_element_by_id(&document, id).unwrap();
    }

    #[wasm_bindgen_test]
    fn should_get_element_by_id_dyn() {
        let id = "id";
        let document = get_new_document();
        let element = document.create_element("p").unwrap();
        element.set_id(id);

        document.get_root_node().append_child(&element).unwrap();
        get_element_by_id_dyn::<Element>(&document, id).unwrap();
    }

    #[wasm_bindgen_test]
    #[should_panic(expected = "element does not exist")]
    fn should_not_get_element_by_id_dyn() {
        let id = "id";

        let document = get_new_document();
        get_element_by_id_dyn::<Element>(&document, id).unwrap();
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
            .build(&document, name)
            .unwrap();

        assert_eq!(name, new_element.tag_name());
        assert_eq!(parent, new_element.parent_element().unwrap());
        assert_eq!(inner_html, new_element.inner_html());
    }

    #[wasm_bindgen_test]
    fn should_create_element_without_parent_or_text() {
        let document = Document::new().unwrap();
        let name = "p";

        let new_element = create_element(&document, name).unwrap();

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
            .build(&document, name)
            .unwrap();

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

        append_child(&container, &child).unwrap();
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
        set_attribute(&element, key, value).unwrap();
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
        remove_attribute(&element, key).unwrap();
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
