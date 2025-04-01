use crate::component::alert::unwrap_or_alert;
use crate::component::login_form::add_enter_listener_on_form;
use crate::component::stepper::add_step;
use web_sys::Document;

mod check;
mod credentials;
pub mod load_members_from_csv;
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
    add_enter_listener_on_form(document, "login-form-fileo");
}
