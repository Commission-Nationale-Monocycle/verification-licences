use crate::utils::{
    append_child, create_element, get_element_by_id, query_selector_single_element,
};
use wasm_bindgen::JsCast;
use web_sys::{Document, DocumentFragment, Element, HtmlTemplateElement};

/// Retrieve a template from the document and make it available as an Element.
pub fn get_template(document: &Document, template_id: &str) -> Element {
    let container = create_element(document, "div", None, None);

    let template = get_element_by_id(document, template_id)
        .dyn_into::<HtmlTemplateElement>()
        .unwrap_or_else(|error| panic!("Couldn't retrieve template `{template_id}`: {error:?}"))
        .content()
        .clone_node_with_deep(true)
        .unwrap_or_else(|error| panic!("Couldn't clone node: {error:?}"))
        .dyn_into::<DocumentFragment>()
        .unwrap_or_else(|error| panic!("Couldn't cast to DocumentFragment: {error:?}"));

    // Making the template available as an element
    append_child(&container, &template);

    // Retrieving the template as it is the only element of the container
    query_selector_single_element(document, &container, "*")
}
