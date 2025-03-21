use crate::Result;
use crate::utils::{add_class, create_element};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element, HtmlElement};

/// Definition of an element in an accordion.
/// `id` should be unique in the accordion for it to work.
pub struct AccordionElement {
    pub id: String,
    title: Element,
    body: Element,
    active: bool,
}

/// Create an accordion based on given items.
/// If sections have to stay opened when another one is opened, set `always_open` to `true`.
#[allow(dead_code)]
pub fn create_accordion(
    document: &Document,
    items: &[AccordionElement],
    always_open: bool,
) -> Result<HtmlElement> {
    let accordion = create_element(document, "div")?.dyn_into::<HtmlElement>()?;
    add_class(&accordion, "accordion");

    let mut accordion_items = vec![];

    for item in items {
        let (title, body, icon) = create_accordion_item(document, item)?;
        accordion.append_child(&title)?;
        accordion.append_child(&body)?;

        accordion_items.push(AccordionItem {
            id: item.id.clone(),
            trigger_element: title,
            target_element: body,
            icon_element: Some(icon),
            active: item.active,
        });
    }

    Accordion::new(
        &accordion,
        accordion_items,
        AccordionOptions::new(always_open),
    );

    Ok(accordion)
}

fn create_accordion_item(
    document: &Document,
    item: &AccordionElement,
) -> Result<(HtmlElement, HtmlElement, HtmlElement)> {
    let icon = {
        let icon = document.create_element("div")?;
        icon.set_inner_html(r#"<svg class="accordion-title-icon" data-accordion-icon xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 10 6"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5 5 1 1 5"/></svg>"#);
        icon
    }.dyn_into::<HtmlElement>()?;

    let title = {
        let title = create_element(document, "h2")?;
        title.append_child(&item.title)?;

        let button = create_element(document, "div")?;
        add_class(&button, "accordion-title-container");
        button.append_child(&title)?;
        button.append_child(&icon)?;

        button
    }
    .dyn_into::<HtmlElement>()?;

    let body = {
        let body = create_element(document, "div")?;
        body.append_child(&item.body)?;
        add_class(&body, "accordion-body-container");

        if !item.active {
            add_class(&body, "hidden");
        }

        body
    }
    .dyn_into::<HtmlElement>()?;

    Ok((title, body, icon))
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct AccordionItem {
    id: String,
    trigger_element: HtmlElement,
    target_element: HtmlElement,
    icon_element: Option<HtmlElement>,
    active: bool,
}

#[wasm_bindgen]
impl AccordionItem {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter, js_name = "triggerEl")]
    pub fn trigger_element(&self) -> HtmlElement {
        self.trigger_element.clone()
    }

    #[wasm_bindgen(getter, js_name = "targetEl")]
    pub fn target_element(&self) -> HtmlElement {
        self.target_element.clone()
    }

    #[wasm_bindgen(getter, js_name = "iconEl")]
    pub fn icon_element(&self) -> Option<HtmlElement> {
        self.icon_element.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn active(&self) -> bool {
        self.active
    }

    #[wasm_bindgen(setter, js_name = "active")]
    pub fn set_active(&mut self, active: bool) {
        self.active = active
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    type AccordionOptions;

    #[wasm_bindgen(constructor)]
    fn new(#[wasm_bindgen(js_name = "alwaysOpen")] always_open: bool) -> AccordionOptions;

    #[wasm_bindgen(js_namespace = window, js_name = Flowbite)]
    type Accordion;

    #[wasm_bindgen(constructor)]
    fn new(
        #[wasm_bindgen(js_name = "accordionEl")] accordion: &HtmlElement,
        #[wasm_bindgen(js_name = "items")] accordion_items: Vec<AccordionItem>,
        #[wasm_bindgen(js_name = "options")] options: AccordionOptions,
    ) -> Accordion;

}
