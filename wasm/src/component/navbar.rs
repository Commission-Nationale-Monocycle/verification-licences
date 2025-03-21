use crate::Result;
use crate::utils::{
    get_element_by_id, get_url_without_query, query_selector_single_element, set_attribute,
};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlAnchorElement};

pub fn init_navbar(document: &Document) -> Result<()> {
    let url = get_url_without_query()?;

    let nav_list = get_element_by_id(document, "nav-list")?;
    let items = nav_list.children();
    for i in 0..items.length() {
        let link_element = query_selector_single_element(&items.get_with_index(i).unwrap(), "a")?
            .dyn_into::<HtmlAnchorElement>()?;
        let href = link_element.href();
        if href == url {
            set_attribute(&link_element, "aria-current", "page")?;
        }
    }

    Ok(())
}
