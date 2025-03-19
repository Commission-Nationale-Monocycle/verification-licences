mod alert;
mod card_creator;
mod check_memberships;
mod error;
mod fileo;
mod json;
mod navbar;
mod stepper;
mod template;
mod uda;
mod user_interface;
mod utils;
mod web;

use crate::alert::{unwrap_or_alert, unwrap_without_alert};
use crate::error::Error;
use crate::fileo::login::init_login_form_fileo;
use crate::utils::get_document;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn run() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let document = &unwrap_without_alert(get_document());
    unwrap_or_alert(navbar::init_navbar(document));

    init_login_form_fileo(document);
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
