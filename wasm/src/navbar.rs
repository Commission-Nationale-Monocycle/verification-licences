use crate::alert::{AlertLevel, create_alert};
use crate::utils::{
    get_element_by_id, get_url_without_query, query_selector_single_element, set_attribute,
};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlAnchorElement};

pub fn init_navbar(document: &Document) {
    let url = get_url_without_query();

    let nav_list = get_element_by_id(document, "nav_list");
    let items = nav_list.children();
    for i in 0..items.length() {
        let link_element =
            query_selector_single_element(document, &items.get_with_index(i).unwrap(), "a")
                .dyn_into::<HtmlAnchorElement>()
                .unwrap_or_else(|error| {
                    create_alert(
                        document,
                        "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                        AlertLevel::Error,
                    );
                    panic!("Couldn't find link element: {error:?}");
                });
        let href = link_element.href();
        if href == url {
            set_attribute(&link_element, "aria-current", "page");
        }
    }
}
