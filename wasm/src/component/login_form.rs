use crate::fileo::login::login;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, KeyboardEvent};

/// Add a simple event handler to allow submitting the login form using any of the Enter keys.
pub fn init_login_form(document: &Document, login_form_class: &str) {
    let forms = document.get_elements_by_class_name(login_form_class);
    let closure = Closure::wrap(Box::new(|e: KeyboardEvent| {
        spawn_local(async move {
            on_key_down_in_login_form(&e).await;
        });
    }) as Box<dyn Fn(_)>);
    for i in 0..forms.length() {
        let form = forms.item(i).unwrap();
        form.add_event_listener_with_event_listener("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
    }
    closure.forget();
}

/// Simple event handler to allow submitting the login form using any of the Enter keys.
async fn on_key_down_in_login_form(event: &KeyboardEvent) {
    let code = event.code();
    if code == "Enter" || code == "NumpadEnter" {
        login().await;
    }
}
