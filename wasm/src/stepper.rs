use crate::alert::{AlertLevel, create_alert};
use crate::utils::{add_class, remove_class};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Document;

#[wasm_bindgen]
pub fn next_step(document: &Document) {
    let stepper = document
        .get_elements_by_class_name("stepper")
        .get_with_index(0)
        .unwrap();
    let step_list = stepper.get_elements_by_tag_name("li");
    let steps = document.get_elements_by_class_name("step");
    if steps.length() != step_list.length() {
        create_alert(
            document,
            "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
            AlertLevel::Error,
        );
        panic!("Different number of steps in stepper and main article!");
    }

    let mut current_step_index = u32::MAX;
    for i in 0..step_list.length() {
        let stepper_element = step_list.get_with_index(i).unwrap();
        let step = steps.get_with_index(i).unwrap();
        if stepper_element
            .class_name()
            .split(" ")
            .any(|class| class == "stepper-current-step")
        {
            remove_class(&stepper_element, "stepper-current-step");
            add_class(&stepper_element, "stepper-validated-step");

            remove_class(&step, "current-step");

            current_step_index = i;
        }

        if current_step_index != u32::MAX && i == current_step_index + 1 {
            add_class(&stepper_element, "stepper-current-step");
            add_class(&step, "current-step");
        }
    }
}
