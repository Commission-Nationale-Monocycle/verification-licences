use crate::card_creator::CardCreator;
use crate::utils::{create_element, create_element_with_class, create_element_with_classes};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use web_sys::{Document, Element};

#[derive(Debug, Getters, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct MemberToCheck {
    membership_num: String,
    name: String,
    firstname: String,
}

impl MemberToCheck {
    pub fn new(membership_num: String, name: String, firstname: String) -> Self {
        Self {
            membership_num,
            name,
            firstname,
        }
    }
}

impl CardCreator for MemberToCheck {
    fn create_card(&self, document: &Document) -> Element {
        let membership_num = format!("Numéro d'adhésion : {}", self.membership_num());
        let name = format!("Nom : {}", self.name());
        let firstname = format!("Prénom : {}", self.firstname());

        let container =
            create_element_with_classes(document, "div", None, None, &["flex-shrink-0", "m-2"]);
        create_element_with_class(
            document,
            "div",
            Some(&container),
            Some("Membre à vérifier"),
            "font-semibold",
        );
        create_element(document, "div", Some(&container), Some(&membership_num));
        create_element(document, "div", Some(&container), Some(&name));
        create_element(document, "div", Some(&container), Some(&firstname));

        container
    }
}

impl PartialOrd for MemberToCheck {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemberToCheck {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name != other.name {
            self.name.cmp(other.name())
        } else if self.firstname != other.firstname {
            self.firstname().cmp(other.firstname())
        } else {
            self.membership_num.cmp(&other.membership_num)
        }
    }
}