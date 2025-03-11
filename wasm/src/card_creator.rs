use crate::utils::{
    append_child, create_element, get_element_by_id, query_selector_single_element,
};
use MemberStatus::Unknown;
use dto::checked_member::MemberStatus::{Expired, UpToDate};
use dto::checked_member::{CheckedMember, MemberStatus};
use dto::member_to_check::MemberToCheck;
use wasm_bindgen::JsCast;
use web_sys::{Document, DocumentFragment, Element, HtmlAnchorElement, HtmlTemplateElement};

pub const EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME: &str = "checked-member-expired";

pub fn create_card_for_member_to_check(
    document: &Document,
    member_to_check: &MemberToCheck,
) -> Element {
    let container = create_element(document, "div", None, None);

    let card_template = get_member_to_check_template(document);
    append_child(&container, &card_template);

    let card = query_selector_single_element(document, &container, "div");

    query_selector_single_element(document, &card, ".member-to-check-membership-number")
        .set_inner_html(member_to_check.membership_num());
    query_selector_single_element(document, &card, ".member-to-check-name")
        .set_inner_html(member_to_check.name());
    query_selector_single_element(document, &card, ".member-to-check-firstname")
        .set_inner_html(member_to_check.firstname());

    card
}

pub fn create_card_for_checked_member(
    document: &Document,
    checked_member: &CheckedMember,
) -> Element {
    let container = create_element(document, "div", None, None);

    let status = checked_member.compute_member_status();
    let card_template = get_checked_member_template(document, &status);
    append_child(&container, &card_template);

    let card = query_selector_single_element(document, &container, "div");

    query_selector_single_element(document, &card, ".member-to-check-membership-number")
        .set_inner_html(checked_member.member_to_check().membership_num());
    query_selector_single_element(document, &card, ".member-to-check-name")
        .set_inner_html(checked_member.member_to_check().name());
    query_selector_single_element(document, &card, ".member-to-check-firstname")
        .set_inner_html(checked_member.member_to_check().firstname());

    if status == UpToDate || status == Expired {
        let membership = checked_member.membership().as_ref().unwrap();
        query_selector_single_element(document, &card, ".membership-name")
            .set_inner_html(membership.name());
        query_selector_single_element(document, &card, ".membership-firstname")
            .set_inner_html(membership.firstname());
        query_selector_single_element(document, &card, ".membership-end-date")
            .set_inner_html(&membership.end_date().format("%d/%m/%Y").to_string());
        let email_address_container =
            query_selector_single_element(document, &card, "a.membership-email-address")
                .dyn_into::<HtmlAnchorElement>()
                .unwrap();
        email_address_container.set_inner_html(&membership.email_address());
        email_address_container.set_href(&format!("mailto:{}", &membership.email_address()));
    }

    card
}

fn get_member_to_check_template(document: &Document) -> DocumentFragment {
    get_element_by_id(document, "member_to_check")
        .dyn_into::<HtmlTemplateElement>()
        .unwrap_or_else(|error| panic!("Couldn't retrieve member_to_check template: {error:?}"))
        .content()
        .clone_node_with_deep(true)
        .unwrap_or_else(|error| panic!("Couldn't clone node: {error:?}"))
        .dyn_into::<DocumentFragment>()
        .unwrap_or_else(|error| panic!("Couldn't cast to DocumentFragment: {error:?}"))
}

fn get_checked_member_template(
    document: &Document,
    member_status: &MemberStatus,
) -> DocumentFragment {
    match member_status {
        UpToDate => get_element_by_id(document, "checked_member_up_to_date"),
        Expired => get_element_by_id(document, "checked_member_expired"),
        Unknown => get_element_by_id(document, "checked_member_unknown"),
    }
    .dyn_into::<HtmlTemplateElement>()
    .unwrap_or_else(|error| panic!("Couldn't retrieve checked_member template: {error:?}"))
    .content()
    .clone_node_with_deep(true)
    .unwrap_or_else(|error| panic!("Couldn't clone node: {error:?}"))
    .dyn_into::<DocumentFragment>()
    .unwrap_or_else(|error| panic!("Couldn't cast to DocumentFragment: {error:?}"))
}
