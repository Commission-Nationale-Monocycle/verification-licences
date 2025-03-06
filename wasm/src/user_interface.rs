use crate::card_creator::CardCreator;
use crate::utils::{
    append_child, clear_element, create_element, get_document, get_element_by_id,
    get_element_by_id_dyn, remove_attribute, set_attribute,
};
use dto::checked_member::CheckedMember;
use dto::member_to_check::MemberToCheck;
use std::collections::BTreeSet;
use web_sys::{Document, Element, HtmlInputElement};

// region Handle "members to check" file
pub fn render_lines(
    document: &Document,
    csv_content: &str,
    members_to_check: &BTreeSet<MemberToCheck>,
    wrong_lines: &[String],
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

fn create_members_to_check_lines(
    document: &Document,
    members_to_check: &[&MemberToCheck],
) -> Vec<Element> {
    members_to_check
        .iter()
        .map(|member_to_check| member_to_check.create_card(document))
        .collect()
}

fn create_wrong_lines(document: &Document, wrong_lines: &[String]) -> Element {
    let parent_text = if wrong_lines.len() == 1 {
        "La ligne suivante contient une ou des erreurs :"
    } else {
        "Les lignes suivantes contiennent une ou des erreurs :"
    };
    let parent = create_element(document, "div", None, Some(parent_text));

    wrong_lines.iter().for_each(|wrong_line| {
        create_element(document, "p", Some(&parent), Some(wrong_line));
    });

    parent
}
// endregion

// region Handle checked members
pub fn handle_checked_members(checked_members: &Vec<CheckedMember>) {
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

pub fn clear_inputs(document: &Document) {
    get_members_to_check_picker(document).set_value("");
    get_members_to_check_hidden_input(document).set_value("");
    render_lines(document, "", &BTreeSet::new(), &[])
}

// endregion
