use web_sys::{Document, Element};
use dto::membership::MembershipDto;
use crate::utils::{create_element, create_element_with_class, create_element_with_classes};

pub trait CardCreator {
    fn create_card(&self, document: &Document) -> Element;
}

pub trait OptionalCardCreator {
    fn create_card_from_optional(element: &Option<&Self>, document: &Document) -> Element;
}

impl OptionalCardCreator for MembershipDto {
    fn create_card_from_optional(element: &Option<&Self>, document: &Document) -> Element {
        let container = create_element_with_classes(
            document,
            "div",
            None,
            None,
            &["flex", "flex-col", "flex-shrink-0", "justify-center", "m-2"],
        );
        if let Some(membership_dto) = element {
            let name = format!("Nom : {}", membership_dto.name());
            let firstname = format!("Prénom : {}", membership_dto.firstname());
            let end_date = format!(
                "Fin de l'adhésion : {}",
                membership_dto.end_date().format("%d/%m/%Y")
            );
            let email_address = format!("Adresse mail : {}", membership_dto.email_address());

            create_element_with_class(
                document,
                "div",
                Some(&container),
                Some("Membre associé au numéro d'adhésion fourni"),
                "font-semibold",
            );
            create_element(document, "div", Some(&container), Some(&name));
            create_element(document, "div", Some(&container), Some(&firstname));
            create_element(document, "div", Some(&container), Some(&end_date));
            create_element(document, "div", Some(&container), Some(&email_address));
        } else {
            create_element_with_class(
                document,
                "div",
                Some(&container),
                Some("Aucune adhésion trouvée"),
                "font-semibold",
            );
        }

        container
    }
}