use crate::alert::{unwrap_or_alert, unwrap_without_alert};
use crate::stepper::add_step;
use crate::utils::{get_document, get_element_by_id};

mod credentials;
pub mod login;
mod update_list;

pub fn init_fileo_page() {
    let document = unwrap_without_alert(get_document());
    if let Ok(stepper) = get_element_by_id(&document, "stepper") {
        unwrap_or_alert(add_step(&document, &stepper, "Sélection"));
        unwrap_or_alert(add_step(&document, &stepper, "Vérification"));
        unwrap_or_alert(add_step(&document, &stepper, "Notification"));
    }
}
