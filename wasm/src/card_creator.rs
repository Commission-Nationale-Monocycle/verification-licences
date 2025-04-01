use crate::Result;
use crate::error::Error;
use crate::template::get_template;
use crate::utils::{
    add_class, append_child, create_element, query_selector_single_element, set_attribute,
};
use dto::checked_member::{CheckResult, CheckedMember};
use dto::member_to_check::MemberToCheck;
use dto::membership::Membership;
use dto::membership_status::{MemberStatus, compute_member_status};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlAnchorElement, HtmlInputElement};

pub fn create_card_for_member_to_check(
    document: &Document,
    member_to_check: &impl MemberToCheck,
) -> Result<Element> {
    let element = get_member_to_check_template(document)?;
    query_selector_single_element(&element, ".membership-number")?.set_inner_html(
        &member_to_check
            .membership_num()
            .clone()
            .unwrap_or("Non renseigné".to_string()),
    );

    let last_name = member_to_check.last_name();
    let first_name = member_to_check.first_name();
    if let Some(last_name) = last_name {
        if let Some(first_name) = first_name {
            query_selector_single_element(&element, ".name")?.set_inner_html(last_name.as_str());
            query_selector_single_element(&element, ".first-name")?
                .set_inner_html(first_name.as_str());
        }
    } else if let Some(identity) = member_to_check.identity() {
        query_selector_single_element(&element, ".identity")?.set_inner_html(identity.as_str());
    }
    if let Some(club) = member_to_check.club() {
        let club_element = query_selector_single_element(&element, ".club")?;
        club_element.set_inner_html(&club);
    }
    if let Some(email_address) = member_to_check.email() {
        let email_address_container =
            query_selector_single_element(&element, ".email-address-container")?;
        let email_address_element = create_element(document, "a")?;
        add_class(&email_address_element, "email-address");
        email_address_element.set_inner_html(email_address.as_str());
        set_attribute(
            &email_address_element,
            "href",
            &format!("mailto:{email_address}"),
        )?;
        append_child(&email_address_container, &email_address_element)?;
    }
    if let Some(confirmed) = member_to_check.confirmed() {
        let confirmed_container = query_selector_single_element(&element, ".confirmed")?;
        if confirmed {
            confirmed_container.set_inner_html("✔");
        } else {
            confirmed_container.set_inner_html("❌");
        }
    }

    if let Some(id) = member_to_check.id() {
        let uda_id_element = query_selector_single_element(&element, ".uda-id")?
            .dyn_into::<HtmlInputElement>()
            .map_err(Error::from)?;
        uda_id_element.set_value(id.to_string().as_str());
    }

    Ok(element)
}

pub fn create_card_for_checked_member(
    document: &Document,
    checked_member: &CheckedMember<impl MemberToCheck>,
) -> Result<Element> {
    let status = checked_member.compute_member_status();
    let checked_member_card_template = get_checked_member_template(document)?;

    let member_card = create_card_for_member_to_check(document, checked_member.member_to_check())?;
    append_child(&checked_member_card_template, &member_card)?;

    let membership_card = create_membership_card(document, checked_member.membership(), &status)?;
    append_child(&checked_member_card_template, &membership_card)?;

    Ok(checked_member_card_template)
}

fn create_membership_card(
    document: &Document,
    check_result: &CheckResult,
    status: &MemberStatus,
) -> Result<Element> {
    let card = get_membership_template(document, status)?;

    match &check_result {
        CheckResult::Match(membership) | CheckResult::PartialMatch(membership) => {
            if *status == MemberStatus::UpToDate || *status == MemberStatus::Expired {
                query_selector_single_element(&card, ".membership-name")?
                    .set_inner_html(membership.name());
                query_selector_single_element(&card, ".membership-first-name")?
                    .set_inner_html(membership.first_name());
                query_selector_single_element(&card, ".membership-end-date")?
                    .set_inner_html(&membership.end_date().format("%d/%m/%Y").to_string());
                let email_address_container =
                    query_selector_single_element(&card, "a.membership-email-address")?
                        .dyn_into::<HtmlAnchorElement>()?;
                email_address_container.set_inner_html(membership.email_address());
                email_address_container
                    .set_href(&format!("mailto:{}", &membership.email_address()));
            }

            if matches!(check_result, CheckResult::PartialMatch(_)) {
                add_class(&card, "membership-partial-match");
            }
        }
        CheckResult::NoMatch => {}
    }
    Ok(card)
}

pub fn create_known_membership_card(
    document: &Document,
    membership: &Membership,
) -> Result<Element> {
    let status = compute_member_status(Some(membership));

    let card = get_membership_template(document, &status)?;

    query_selector_single_element(&card, ".membership-name")?.set_inner_html(membership.name());
    query_selector_single_element(&card, ".membership-first-name")?
        .set_inner_html(membership.first_name());
    query_selector_single_element(&card, ".membership-end-date")?
        .set_inner_html(&membership.end_date().format("%d/%m/%Y").to_string());
    query_selector_single_element(&card, ".membership-club")?.set_inner_html(membership.club());
    let email_address_container =
        query_selector_single_element(&card, "a.membership-email-address")?
            .dyn_into::<HtmlAnchorElement>()?;
    email_address_container.set_inner_html(membership.email_address());
    email_address_container.set_href(&format!("mailto:{}", &membership.email_address()));

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
        MemberStatus::UpToDate => get_template(document, "membership-up-to-date"),
        MemberStatus::Expired => get_template(document, "membership-expired"),
        MemberStatus::Unknown => get_template(document, "membership-unknown"),
    }
}
