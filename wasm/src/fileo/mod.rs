use crate::alert::unwrap_or_alert;
use crate::login_form::init_login_form;
use crate::stepper::add_step;
use web_sys::Document;

mod credentials;
pub mod login;
mod update_list;

pub fn init_fileo_page(document: &Document) {
    if let Some(stepper) = document
        .get_elements_by_class_name("stepper")
        .get_with_index(0)
    {
        unwrap_or_alert(add_step(document, &stepper, "Sélection"));
        unwrap_or_alert(add_step(document, &stepper, "Vérification"));
        unwrap_or_alert(add_step(document, &stepper, "Notification"));
    }
    init_login_form(document, "login-form-uda");
}
