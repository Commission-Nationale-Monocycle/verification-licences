mod check;
mod confirm;
mod credentials;
mod import_from_uda;
mod update_instances_list;

use crate::alert::unwrap_or_alert;
use crate::login_form::init_login_form;
use crate::stepper::add_step;
use web_sys::Document;

pub fn init_uda_page(document: &Document) {
    if let Some(stepper) = document
        .get_elements_by_class_name("stepper")
        .get_with_index(0)
    {
        unwrap_or_alert(add_step(document, &stepper, "Import"));
        unwrap_or_alert(add_step(document, &stepper, "Participants"));
        unwrap_or_alert(add_step(document, &stepper, "Vérification"));
        unwrap_or_alert(add_step(document, &stepper, "Notification"));
    }

    init_login_form(document, "login-form-uda");
}
