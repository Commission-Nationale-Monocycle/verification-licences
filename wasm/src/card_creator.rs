use crate::Result;
use crate::alert::{AlertLevel, create_alert};
use crate::error::Error;
use crate::template::get_template;
use crate::utils::{append_child, create_element, query_selector_single_element};
use MemberStatus::Unknown;
use dto::checked_member::MemberStatus::{Expired, UpToDate};
use dto::checked_member::{CheckedMember, MemberStatus};
use dto::member_to_check::MemberToCheck;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlAnchorElement};

pub const EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME: &str = "checked-member-expired";

pub fn create_card_for_member_to_check(
    document: &Document,
    member_to_check: &MemberToCheck,
) -> Result<Element> {
    let container = create_element(document, "div")?;

    let card_template = get_member_to_check_template(document)?;
    append_child(&container, &card_template)?;

    let card = query_selector_single_element(&container, "div")?;

    let membership_num = member_to_check
        .membership_num()
        .clone()
        .unwrap_or("Aucun numéro de licence n'a été fourni".to_owned());
    query_selector_single_element(&card, ".member-to-check-membership-number")?
        .set_inner_html(&membership_num);
    query_selector_single_element(&card, ".member-to-check-name")?
        .set_inner_html(member_to_check.name());
    query_selector_single_element(&card, ".member-to-check-firstname")?
        .set_inner_html(member_to_check.firstname());

    Ok(card)
}

pub fn create_card_for_checked_member(
    document: &Document,
    checked_member: &CheckedMember,
) -> Result<Element> {
    let container = create_element(document, "div")?;

    let status = checked_member.compute_member_status();
    let card_template = get_checked_member_template(document, &status)?;
    append_child(&container, &card_template)?;

    let card = query_selector_single_element(&container, "div")?;

    let membership_num = checked_member.member_to_check().membership_num().clone().ok_or_else(|| {
        create_alert("Un membre a été vérifié, sans pour autant avoir de numéro de licence. Ce cas ne devrait pas se produire.", AlertLevel::Error);
        Error::new("Membership does not provide number. Can't display card.".to_owned())
    })?;
    query_selector_single_element(&card, ".member-to-check-membership-number")?
        .set_inner_html(&membership_num);
    query_selector_single_element(&card, ".member-to-check-name")?
        .set_inner_html(checked_member.member_to_check().name());
    query_selector_single_element(&card, ".member-to-check-firstname")?
        .set_inner_html(checked_member.member_to_check().firstname());

    if status == UpToDate || status == Expired {
        let membership = checked_member.membership().as_ref().unwrap();
        query_selector_single_element(&card, ".membership-name")?.set_inner_html(membership.name());
        query_selector_single_element(&card, ".membership-firstname")?
            .set_inner_html(membership.firstname());
        query_selector_single_element(&card, ".membership-end-date")?
            .set_inner_html(&membership.end_date().format("%d/%m/%Y").to_string());
        let email_address_container =
            query_selector_single_element(&card, "a.membership-email-address")?
                .dyn_into::<HtmlAnchorElement>()?;
        email_address_container.set_inner_html(membership.email_address());
        email_address_container.set_href(&format!("mailto:{}", &membership.email_address()));
    }

    Ok(card)
}

fn get_member_to_check_template(document: &Document) -> Result<Element> {
    get_template(document, "member-to-check")
}

fn get_checked_member_template(
    document: &Document,
    member_status: &MemberStatus,
) -> Result<Element> {
    match member_status {
        UpToDate => get_template(document, "checked-member-up-to-date"),
        Expired => get_template(document, "checked-member-expired"),
        Unknown => get_template(document, "checked-member-unknown"),
    }
}
