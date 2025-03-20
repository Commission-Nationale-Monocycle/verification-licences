use crate::Result;
use crate::alert::{AlertLevel, create_alert};
use crate::error::{DEFAULT_ERROR_MESSAGE, Error};
use crate::template::get_template;
use crate::utils::{add_class, append_child, remove_class, set_attribute};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Document, Element, HtmlCollection};

pub fn add_step(document: &Document, stepper: &Element, step_name: &str) -> Result<()> {
    let steps = stepper.get_elements_by_tag_name("li");
    let steps_count = steps.length();

    let step_element = get_template(document, "step-template")?;
    if steps_count == 0 {
        add_class(&step_element, "stepper-current-step");
    }
    set_attribute(
        &step_element,
        "onclick",
        &format!("app.go_to_step(document, {steps_count})"),
    )?;
    let step_name_element = step_element
        .get_elements_by_class_name("step-name")
        .get_with_index(0)
        .ok_or_else(|| {
            Error::new(
                DEFAULT_ERROR_MESSAGE.to_string(),
                "Step name element not found".to_owned(),
            )
        })?;
    let current_step_name_content = step_name_element.inner_html();
    step_name_element.set_inner_html(&format!("{current_step_name_content}{step_name}"));

    let step_index_element = step_element
        .get_elements_by_class_name("step-index")
        .get_with_index(0)
        .ok_or_else(|| {
            Error::new(
                DEFAULT_ERROR_MESSAGE.to_string(),
                "Step index element not found".to_owned(),
            )
        })?;
    step_index_element.set_text_content(Some(&format!("{}.", steps_count + 1)));

    append_child(stepper, &step_element)?;

    Ok(())
}

#[wasm_bindgen]
pub fn next_step(document: &Document) {
    let step_list = get_step_list(document);
    let step_elements = get_step_elements(document);
    if step_elements.length() != step_list.length() {
        create_alert(
            "Erreur lors du traitement. Veuillez actualiser la page et réessayer.",
            AlertLevel::Error,
        );
        panic!("Different number of steps in stepper and main article!");
    }

    let mut current_step_index = u32::MAX;
    for i in 0..step_list.length() {
        let stepper_element = step_list.get_with_index(i).unwrap();
        let step_element = step_elements.get_with_index(i).unwrap();
        if is_current_step(&stepper_element) {
            remove_current_step(&stepper_element, &step_element);
            add_class(&stepper_element, "stepper-validated-step");

            current_step_index = i;
        }

        if current_step_index != u32::MAX && i == current_step_index + 1 {
            set_current_step(&stepper_element, &step_element);
        }
    }
}

#[wasm_bindgen]
pub fn go_to_step(document: &Document, clicked_index: u32) {
    let stepper = get_stepper(document);
    let current_step = find_current_step(&stepper);

    if clicked_index < current_step {
        let stepper_list = get_step_list(document);
        let step_elements = get_step_elements(document);

        {
            let current_step = find_current_step(&stepper);
            let current_stepper_element = stepper_list.get_with_index(current_step).unwrap();
            let current_step_element = step_elements.get_with_index(current_step).unwrap();
            remove_current_step(&current_stepper_element, &current_step_element);
        }

        {
            let new_current_stepper_element = stepper_list.get_with_index(clicked_index).unwrap();
            let new_current_step_element = step_elements.get_with_index(clicked_index).unwrap();
            set_current_step(&new_current_stepper_element, &new_current_step_element);
        }

        for i in clicked_index..stepper_list.length() {
            let stepper_element = stepper_list.get_with_index(i).unwrap();
            remove_class(&stepper_element, "stepper-validated-step")
        }
    }
}

fn set_current_step(stepper_element: &Element, step_element: &Element) {
    add_class(stepper_element, "stepper-current-step");
    add_class(step_element, "current-step");
}

fn remove_current_step(stepper_element: &Element, step_element: &Element) {
    remove_class(stepper_element, "stepper-current-step");
    remove_class(step_element, "current-step");
}

fn get_stepper(document: &Document) -> Element {
    document
        .get_elements_by_class_name("stepper")
        .get_with_index(0)
        .unwrap()
}

fn get_step_list(document: &Document) -> HtmlCollection {
    let stepper = get_stepper(document);
    stepper.get_elements_by_tag_name("li")
}

fn get_step_elements(document: &Document) -> HtmlCollection {
    document.get_elements_by_class_name("step")
}

fn is_current_step(stepper_element: &Element) -> bool {
    stepper_element
        .class_name()
        .split(" ")
        .any(|class| class == "stepper-current-step")
}

fn find_current_step(stepper_element: &Element) -> u32 {
    let steps = stepper_element.get_elements_by_tag_name("li");
    for i in 0..steps.length() {
        let step = steps.get_with_index(i).unwrap();
        if is_current_step(&step) {
            return i;
        }
    }

    0
}
