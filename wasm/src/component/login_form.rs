use crate::Result;
use crate::component::alert::unwrap_or_alert;
use crate::utils::{get_body, query_selector_single_element};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Event, KeyboardEvent};

/// Add a simple event handler to allow submitting the form using any of the Enter keys.
pub fn add_enter_listener_on_form(document: &Document, form_class: &str) {
    let forms = document.get_elements_by_class_name(form_class);
    let closure =
        Closure::wrap(
            Box::new(|e: KeyboardEvent| unwrap_or_alert(on_key_down_in_form(&e))) as Box<dyn Fn(_)>,
        );
    for i in 0..forms.length() {
        let form = forms.item(i).unwrap();
        form.add_event_listener_with_event_listener("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
    }
    closure.forget();
}

/// Simple event handler to allow submitting the form using any of the Enter keys.
fn on_key_down_in_form(event: &KeyboardEvent) -> Result<()> {
    let code = event.code();
    if code == "Enter" || code == "NumpadEnter" {
        let button = query_selector_single_element(&get_body()?.into(), ".submit-button")?;
        button.dispatch_event(&Event::new("click")?)?;
    }

    Ok(())
}
