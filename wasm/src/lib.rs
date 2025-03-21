mod card_creator;
mod check_memberships;
mod component;
mod error;
mod fileo;
mod json;
mod template;
mod uda;
mod user_interface;
mod utils;
mod web;

use crate::component::alert::{unwrap_or_alert, unwrap_without_alert};
use crate::component::navbar;
use crate::error::Error;
use crate::fileo::init_fileo_page;
use crate::uda::init_uda_page;
use crate::utils::{get_document, get_element_by_id};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn run() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let document = &unwrap_without_alert(get_document());
    unwrap_or_alert(navbar::init_navbar(document));

    if get_element_by_id(document, "fileo-container").is_ok() {
        init_fileo_page(document);
    } else if get_element_by_id(document, "uda-container").is_ok() {
        init_uda_page(document);
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
