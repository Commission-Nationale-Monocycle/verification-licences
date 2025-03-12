use crate::alert::{AlertLevel, create_alert, unwrap_or_alert, unwrap_without_alert};
use crate::build_client;
use crate::error::Error;
use crate::user_interface::set_loading;
use crate::utils::{
    get_document, get_element_by_id_dyn, get_location, get_value_from_element, get_window,
};
use reqwest::StatusCode;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlInputElement, KeyboardEvent, UrlSearchParams};

/// Add a simple event handler to allow submitting the login form using any of the Enter keys.
pub fn init_login_form_fileo(document: &Document) {
    let forms = document.get_elements_by_class_name("login-form-fileo");
    let closure = Closure::wrap(Box::new(|e: KeyboardEvent| {
        spawn_local(async move {
            on_key_down_in_fileo_login_form(&e).await;
        });
    }) as Box<dyn Fn(_)>);
    for i in 0..forms.length() {
        let form = forms.item(i).unwrap();
        form.add_event_listener_with_event_listener("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
    }
    closure.forget();
}

/// Validate the field, then try to log into Fileo app.
/// If it succeeds, then redirect to the verification page.
#[wasm_bindgen]
pub async fn login() {
    unwrap_or_alert(set_loading(true));

    let document = unwrap_without_alert(get_document());
    let login_field = unwrap_without_alert(get_element_by_id_dyn::<HtmlInputElement>(
        &document, "login",
    ));
    let password_field = unwrap_without_alert(get_element_by_id_dyn::<HtmlInputElement>(
        &document, "password",
    ));

    if !login_field.report_validity() || !password_field.report_validity() {
        unwrap_or_alert(set_loading(false));
        return;
    }

    let login = get_value_from_element(&login_field);
    let password = get_value_from_element(&password_field);

    let client = build_client();
    let origin = unwrap_without_alert(get_window())
        .location()
        .origin()
        .unwrap();
    let url = format!("{origin}/api/fileo/login");
    let body = json!({
        "login": login, "password": password
    })
    .to_string();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| {
            unwrap_or_alert(set_loading(false));
            create_alert(
                "Impossible d'envoyer la requête. Veuillez réessayer.",
                AlertLevel::Error,
            );
            panic!("can't send request: {error:?}")
        });

    let status = response.status();
    if status.is_success() {
        unwrap_or_alert(set_loading(false));
        let location = unwrap_or_alert(get_location());
        let query_params = unwrap_or_alert(location.search().map_err(|error| {
            Error::from_parent(
                "Erreur, veuillez réessayer.".to_owned(),
                Error::new(error.as_string().unwrap()),
            )
        }));
        let query_params = unwrap_or_alert(UrlSearchParams::new_with_str(&query_params).map_err(
            |error| {
                Error::from_parent(
                    "Erreur, veuillez réessayer.".to_owned(),
                    Error::new(error.as_string().unwrap()),
                )
            },
        ));
        let url_to_redirect = if let Some(redirect) = query_params.get("page") {
            redirect
        } else {
            "/check-memberships".to_owned()
        };
        let result = location.set_href(&url_to_redirect);
        if let Err(error) = result {
            create_alert(
                "Erreur lors de la redirection. Veuillez actualiser la page.",
                AlertLevel::Error,
            );
            log::error!("Can't redirect user: {error:?}");
        }
    } else if status == StatusCode::UNAUTHORIZED {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Vos identifiants sont incorrects. Veuillez réessayer.",
            AlertLevel::Error,
        );
    } else {
        unwrap_or_alert(set_loading(false));
        create_alert(
            "Impossible de se connecter. Veuillez réessayer.",
            AlertLevel::Error,
        );
        log::error!("Server error: {}", response.status().as_str());
    }
}

/// Simple event handler to allow submitting the login form using any of the Enter keys.
async fn on_key_down_in_fileo_login_form(event: &KeyboardEvent) {
    let code = event.code();
    if code == "Enter" || code == "NumpadEnter" {
        login().await;
    }
}
