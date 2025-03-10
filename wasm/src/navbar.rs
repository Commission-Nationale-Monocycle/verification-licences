use crate::toast::{ToastLevel, show_toast};
use crate::utils::{get_element_by_id, get_url_without_query, query_selector_single_element};
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
                    show_toast(
                        document,
                        "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
                        ToastLevel::Error,
                    );
                    panic!("Couldn't find link element: {error:?}");
                });
        let href = link_element.href();
        if href == url {
            link_element.set_class_name("block py-2 px-3 text-white bg-blue-700 rounded-sm md:bg-transparent md:text-blue-700 md:p-0 dark:text-white md:dark:text-blue-500");
        } else {
            link_element.set_class_name("block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent");
        }
    }
}
