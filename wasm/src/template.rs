use crate::Result;
use crate::utils::{
    append_child, create_element, get_element_by_id, query_selector_single_element,
};
use wasm_bindgen::JsCast;
use web_sys::{Document, DocumentFragment, Element, HtmlTemplateElement};

/// Retrieve a template from the document and make it available as an Element.
pub fn get_template(document: &Document, template_id: &str) -> Result<Element> {
    let container = create_element(document, "div")?;

    let template = get_element_by_id(document, template_id)?
        .dyn_into::<HtmlTemplateElement>()?
        .content()
        .clone_node_with_deep(true)?
        .dyn_into::<DocumentFragment>()?;

    // Making the template available as an element
    append_child(&container, &template)?;

    // Retrieving the template as it is the only element of the container
    query_selector_single_element(&container, "*")
}
