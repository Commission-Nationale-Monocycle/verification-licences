use crate::Result;
use crate::card_creator::{create_card_for_checked_member, create_card_for_member_to_check};
use crate::utils::{
    ElementBuilder, add_class, append_child, clear_element, get_body, get_document,
    get_element_by_id, get_element_by_id_dyn, remove_attribute, remove_class, set_attribute,
};
use dto::checked_member::CheckedMember;
use dto::checked_member::MemberStatus::Expired;
use dto::member_to_check::MemberToCheck;
use std::collections::BTreeSet;
use web_sys::{Document, Element, HtmlInputElement};

// region Handle "members to check" file
pub fn render_lines(
    document: &Document,
    csv_content: &str,
    members_to_check: &BTreeSet<MemberToCheck>,
    wrong_lines: &[String],
) -> Result<()> {
    let members_to_check_hidden_input = get_members_to_check_hidden_input(document)?;
    let members_to_check_table = get_members_to_check_table(document)?;
    let wrong_lines_paragraph = get_wrong_line_paragraph(document)?;
    let submit_button = get_submit_button(document)?;
    let checked_members_container = get_checked_members_container(document)?;

    clear_element(&members_to_check_table);
    clear_element(&wrong_lines_paragraph);
    clear_element(&checked_members_container);

    if !wrong_lines.is_empty() {
        let wrong_lines_data = create_wrong_lines(document, wrong_lines)?;
        append_child(&wrong_lines_paragraph, &wrong_lines_data)?;
    }
    if !members_to_check.is_empty() {
        let members_to_check = members_to_check.iter().collect::<Vec<_>>();
        let lines = create_members_to_check_lines(document, &members_to_check)?;
        for line in lines {
            append_child(&members_to_check_table, &line)?;
        }
        set_attribute(&members_to_check_hidden_input, "value", csv_content)?;
        remove_attribute(&submit_button, "disabled")?;
    } else {
        set_attribute(&submit_button, "disabled", "true")?;
    }

    Ok(())
}

fn create_members_to_check_lines(
    document: &Document,
    members_to_check: &[&MemberToCheck],
) -> Result<Vec<Element>> {
    members_to_check
        .iter()
        .map(|member_to_check| create_card_for_member_to_check(document, member_to_check))
        .collect()
}

fn create_wrong_lines(document: &Document, wrong_lines: &[String]) -> Result<Element> {
    let parent_text = if wrong_lines.len() == 1 {
        "La ligne suivante contient une ou des erreurs :"
    } else {
        "Les lignes suivantes contiennent une ou des erreurs :"
    };
    let parent = ElementBuilder::default()
        .inner_html(parent_text)
        .build(document, "div")?;

    for wrong_line in wrong_lines {
        ElementBuilder::default()
            .parent(&parent)
            .inner_html(wrong_line)
            .build(document, "p")?;
    }

    Ok(parent)
}
// endregion

// region Handle checked members
pub fn handle_checked_members(checked_members: &Vec<CheckedMember>) -> Result<()> {
    let document = get_document()?;
    let parent = get_element_by_id(&document, "checked-members")?;
    clear_element(&parent);
    for checked_member in checked_members {
        let card = create_card_for_checked_member(&document, checked_member)?;
        append_child(&parent, &card)?;
    }

    if checked_members
        .iter()
        .any(|checked_member| checked_member.compute_member_status() == Expired)
    {
        let write_email_container = get_write_email_container(&document)?;
        remove_class(&write_email_container, "hidden");
    }

    Ok(())
}
// endregion

// region Get parts of the document
fn get_members_to_check_hidden_input(document: &Document) -> Result<HtmlInputElement> {
    get_element_by_id_dyn(document, "members-to-check")
}

fn get_members_to_check_table(document: &Document) -> Result<Element> {
    get_element_by_id(document, "members-to-check-table")
}

fn get_wrong_line_paragraph(document: &Document) -> Result<Element> {
    get_element_by_id(document, "wrong-lines-paragraph")
}

fn get_submit_button(document: &Document) -> Result<Element> {
    get_element_by_id(document, "submit-members")
}

pub fn get_checked_members_container(document: &Document) -> Result<Element> {
    get_element_by_id(document, "checked-members")
}

fn get_write_email_container(document: &Document) -> Result<Element> {
    get_element_by_id(document, "write-email-container")
}

pub fn get_email_subject(document: &Document) -> Result<String> {
    get_element_by_id_dyn::<HtmlInputElement>(document, "email-subject")
        .map(|element| element.value())
}

pub fn get_email_body(document: &Document) -> Result<String> {
    get_element_by_id(document, "email-body").map(|element| element.inner_html())
}
// endregion

pub fn set_loading(loading: bool) -> Result<()> {
    let body = get_body()?;
    if loading {
        add_class(&body, "loading");
    } else {
        remove_class(&body, "loading");
    }

    Ok(())
}
