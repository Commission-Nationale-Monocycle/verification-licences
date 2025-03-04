mod card_creator;
mod checked_member;
mod member_to_check;
mod membership_dto;
mod utils;

use crate::card_creator::CardCreator;
use crate::checked_member::CheckedMember;
use crate::member_to_check::MemberToCheck;
use crate::utils::{
    append_child, clear_element, get_document, get_element_by_id, get_element_by_id_dyn,
    get_value_from_input, get_window, remove_attribute, set_attribute,
};
use csv::StringRecord;
use reqwest::Client;
use std::collections::BTreeSet;
use utils::create_element;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, Element, Event, HtmlFormElement, HtmlInputElement};

#[wasm_bindgen(start)]
fn run() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    add_submit_event_listener_to_form();
}

// region Handle "members to check" file
#[wasm_bindgen]
pub async fn handle_members_to_check_file(input: HtmlInputElement) -> Result<(), JsValue> {
    let document = get_document();

    let csv_file = input
        .files()
        .expect("no files")
        .get(0)
        .expect("file should be accessible");

    let promise = csv_file.text();
    let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let csv_content = text_jsvalue
        .as_string()
        .expect("csv file should contain only valid UTF-8 characters");

    let (members_to_check, wrong_lines) = parse_csv(&csv_content).await?;

    render_lines(&document, &csv_content, &members_to_check, &wrong_lines);

    Ok(())
}

fn render_lines(
    document: &Document,
    csv_content: &str,
    members_to_check: &BTreeSet<MemberToCheck>,
    wrong_lines: &[StringRecord],
) {
    let members_to_check_hidden_input = get_members_to_check_hidden_input(document);
    let members_to_check_table = get_members_to_check_table(document);
    let wrong_lines_paragraph = get_element_by_id(document, "wrong_lines_paragraph");
    let submit_button = get_element_by_id(document, "submit_members");

    clear_element(&members_to_check_table);
    clear_element(&wrong_lines_paragraph);

    if !wrong_lines.is_empty() {
        let wrong_lines_data = create_wrong_lines(document, wrong_lines);
        append_child(&wrong_lines_paragraph, &wrong_lines_data);
    }
    if !members_to_check.is_empty() {
        let members_to_check = members_to_check.iter().collect::<Vec<_>>();
        let lines = create_members_to_check_lines(document, &members_to_check);
        lines.iter().for_each(|line| {
            append_child(&members_to_check_table, line);
        });
        set_attribute(&members_to_check_hidden_input, "value", csv_content);
        remove_attribute(&submit_button, "disabled");
    } else {
        set_attribute(&submit_button, "disabled", "true");
    }
}

async fn parse_csv(
    csv_content: &str,
) -> Result<(BTreeSet<MemberToCheck>, Vec<StringRecord>), JsValue> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(csv_content.as_bytes());

    let mut members_to_check = BTreeSet::new();
    let mut wrong_lines = vec![];

    reader.records().for_each(|record| {
        if let Ok(record) = record {
            if record.len() != 3 {
                wrong_lines.push(record);
            } else {
                members_to_check.insert(MemberToCheck::new(
                    record.get(0).unwrap().to_owned(),
                    record.get(1).unwrap().to_owned(),
                    record.get(2).unwrap().to_owned(),
                ));
            }
        } else {
            println!("Error while reading member");
        };
    });

    Ok((members_to_check, wrong_lines))
}

fn create_members_to_check_lines(
    document: &Document,
    members_to_check: &[&MemberToCheck],
) -> Vec<Element> {
    members_to_check
        .iter()
        .map(|member_to_check| member_to_check.create_card(document))
        .collect()
}

fn create_wrong_lines(document: &Document, wrong_lines: &[StringRecord]) -> Element {
    let parent_text = if wrong_lines.len() == 1 {
        "La ligne suivante contient une ou des erreurs :"
    } else {
        "Les lignes suivantes contiennent une ou des erreurs :"
    };
    let parent = create_element(document, "div", None, Some(parent_text));

    wrong_lines.iter().for_each(|wrong_line| {
        let line = wrong_line.iter().collect::<Vec<&str>>().join(";");
        create_element(document, "p", Some(&parent), Some(&line));
    });

    parent
}
// endregion

// region Handle form submission
fn add_submit_event_listener_to_form() {
    let document = get_document();
    let form = get_element_by_id_dyn::<HtmlFormElement>(&document, "check_members_form");
    let closure = Closure::wrap(Box::new(|e: Event| {
        {
            spawn_local(async move {
                handle_form_submission(e).await;
            });
        }
    }) as Box<dyn Fn(_)>);
    form.add_event_listener_with_event_listener("submit", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

async fn handle_form_submission(e: Event) {
    e.prevent_default();
    let document = get_document();
    let members_to_check_input = get_value_from_input(&document, "members_to_check");

    let client = build_client();

    let origin = get_window().location().origin().unwrap();
    let url = format!("{origin}/api/members/check");
    let body = format!("members_to_check={members_to_check_input}");
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("can't send request");

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        let text = response.text().await.expect("can't get text");
        log::info!("{text}");
        let checked_members: Vec<CheckedMember> =
            serde_json::from_str(&text).expect("can't deserialize checked members");
        handle_checked_members(&checked_members);
        clear_inputs(&document);
    } else {
        log::error!("Server error: {}", response.status().as_str())
    }
}

fn build_client() -> Client {
    Client::builder().build().expect("could not build client")
}
// endregion

// region Handle checked members
fn handle_checked_members(checked_members: &Vec<CheckedMember>) {
    let document = get_document();
    let parent = get_element_by_id(&document, "checked_members");
    for checked_member in checked_members {
        let card = checked_member.create_card(&document);
        append_child(&parent, &card);
    }
}
// endregion

// region Get parts of the document
fn get_members_to_check_hidden_input(document: &Document) -> HtmlInputElement {
    get_element_by_id_dyn(document, "members_to_check")
}

fn get_members_to_check_picker(document: &Document) -> HtmlInputElement {
    get_element_by_id_dyn(document, "members_to_check_picker")
}

fn get_members_to_check_table(document: &Document) -> Element {
    get_element_by_id(document, "members_to_check_table")
}
// endregion

fn clear_inputs(document: &Document) {
    get_members_to_check_picker(document).set_value("");
    get_members_to_check_hidden_input(document).set_value("");
    render_lines(document, "", &BTreeSet::new(), &[])
}
