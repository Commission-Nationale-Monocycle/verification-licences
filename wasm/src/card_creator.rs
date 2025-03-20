use crate::Result;
use crate::error::Error;
use crate::template::get_template;
use crate::utils::{append_child, create_element, query_selector_single_element, set_attribute};
use MemberStatus::Unknown;
use dto::checked_member::MemberStatus::{Expired, UpToDate};
use dto::checked_member::{CheckedMember, MemberStatus};
use dto::csv_member::CsvMember;
use dto::membership::Membership;
use dto::uda_member::UdaMember;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlAnchorElement};

pub const EXPIRED_CHECKED_MEMBER_CONTAINER_CLASS_NAME: &str = "checked-member-expired";

pub fn create_card_for_csv_member_to_check(
    document: &Document,
    member_to_check: &CsvMember,
) -> Result<Element> {
    let container = create_element(document, "div")?;

    let card_template = get_member_to_check_template(document)?;
    append_child(&container, &card_template)?;

    let card = query_selector_single_element(&container, "div")?;

    let membership_num = member_to_check.membership_num();
    query_selector_single_element(&card, ".membership-number")?.set_inner_html(membership_num);
    query_selector_single_element(&card, ".name")?.set_inner_html(member_to_check.name());
    query_selector_single_element(&card, ".first-name")?
        .set_inner_html(member_to_check.first_name());

    Ok(card)
}

pub fn create_card_for_csv_checked_member(
    document: &Document,
    checked_member: &CheckedMember<CsvMember>,
) -> Result<Element> {
    let status = checked_member.compute_member_status();
    let checked_member_card_template = get_checked_member_template(document)?;

    let member_card =
        create_card_for_csv_member_to_check(document, checked_member.member_to_check())?;
    append_child(&checked_member_card_template, &member_card)?;

    let membership_card = create_membership_card(document, checked_member.membership(), &status)?;
    append_child(&checked_member_card_template, &membership_card)?;

    Ok(checked_member_card_template)
}

pub fn create_card_for_uda_member_to_check(
    document: &Document,
    member_to_check: &UdaMember,
) -> Result<Element> {
    let element = get_member_to_check_template(document)?;
    query_selector_single_element(&element, ".membership-number")?.set_inner_html(
        &member_to_check
            .membership_number()
            .clone()
            .unwrap_or("Non renseigné".to_string()),
    );
    query_selector_single_element(&element, ".name")?
        .set_inner_html(member_to_check.last_name().as_str());
    query_selector_single_element(&element, ".first-name")?
        .set_inner_html(member_to_check.first_name().as_str());
    let club_element = query_selector_single_element(&element, ".club")?;
    if let Some(club) = member_to_check.club() {
        club_element.set_inner_html(club);
    }
    let email_address_element = query_selector_single_element(&element, ".email-address")?;
    let email_address = member_to_check.email().as_str();
    email_address_element.set_inner_html(email_address);
    set_attribute(
        &email_address_element,
        "href",
        &format!("mailto:{email_address}"),
    )?;

    Ok(element)
}

pub fn create_card_for_uda_checked_member(
    document: &Document,
    checked_member: &CheckedMember<UdaMember>,
) -> Result<Element> {
    let status = checked_member.compute_member_status();
    let checked_member_card_template = get_checked_member_template(document)?;

    let member_card =
        create_card_for_uda_member_to_check(document, checked_member.member_to_check())?;
    append_child(&checked_member_card_template, &member_card)?;

    let membership_card = create_membership_card(document, checked_member.membership(), &status)?;
    append_child(&checked_member_card_template, &membership_card)?;

    Ok(checked_member_card_template)
}

fn create_membership_card(
    document: &Document,
    membership: &Option<Membership>,
    status: &MemberStatus,
) -> Result<Element> {
    let card = get_membership_template(document, status)?;
    if *status == UpToDate || *status == Expired {
        let membership = membership.clone().ok_or_else(Error::default)?;

        query_selector_single_element(&card, ".membership-name")?.set_inner_html(membership.name());
        query_selector_single_element(&card, ".membership-first-name")?
            .set_inner_html(membership.first_name());
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
    get_template(document, "member-to-check-template")
}

fn get_checked_member_template(document: &Document) -> Result<Element> {
    get_template(document, "checked-member")
}

fn get_membership_template(document: &Document, member_status: &MemberStatus) -> Result<Element> {
    match member_status {
        UpToDate => get_template(document, "membership-up-to-date"),
        Expired => get_template(document, "membership-expired"),
        Unknown => get_template(document, "membership-unknown"),
    }
}
