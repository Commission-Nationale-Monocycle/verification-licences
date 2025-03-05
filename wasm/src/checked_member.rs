use crate::card_creator::{CardCreator, OptionalCardCreator};
use crate::utils::{append_child, create_element_with_classes};
use chrono::Utc;
use dto::membership::Membership;
use serde::Deserialize;
use std::cmp::Ordering;
use web_sys::{Document, Element};
use dto::member_to_check::MemberToCheck;

#[derive(Deserialize, PartialEq)]
pub struct CheckedMember {
    member_to_check: MemberToCheck,
    membership_dto: Option<Membership>,
}

impl CardCreator for CheckedMember {
    fn create_card(&self, document: &Document) -> Element {
        let container = create_element_with_classes(
            document,
            "div",
            None,
            None,
            &[
                // Flex
                "flex",
                "flex-col",
                "md:flex-row",
                ".flex-shrink-0",
                // Spacing
                "m-2",
                // Border
                "border-2",
                "rounded-md",
            ],
        );

        let member_to_check_card = self.member_to_check.create_card(document);
        append_child(&container, &member_to_check_card);

        let membership_card =
            Membership::create_card_from_optional(&self.membership_dto.as_ref(), document);
        append_child(&container, &membership_card);

        {
            if let Some(membership_dto) = &self.membership_dto {
                if Utc::now().date_naive() > *membership_dto.end_date() {
                    let classes = format!("{} bg-orange-300", container.class_name());
                    container.set_class_name(&classes);
                }
            } else {
                let classes = format!("{} bg-red-300", container.class_name());
                container.set_class_name(&classes);
            }
        }

        container
    }
}

impl PartialOrd for CheckedMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.membership_dto.is_some() && other.membership_dto.is_none() {
            Some(Ordering::Greater)
        } else if self.membership_dto.is_none() && other.membership_dto.is_some() {
            Some(Ordering::Less)
        } else {
            self.member_to_check.partial_cmp(&other.member_to_check)
        }
    }
}
