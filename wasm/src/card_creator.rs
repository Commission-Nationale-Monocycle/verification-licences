use crate::utils::{create_element, create_element_with_class, create_element_with_classes};
use dto::member_to_check::MemberToCheck;
use dto::membership::Membership;
use web_sys::{Document, Element};

pub trait CardCreator {
    fn create_card(&self, document: &Document) -> Element;
}

pub trait OptionalCardCreator {
    fn create_card_from_optional(element: &Option<&Self>, document: &Document) -> Element;
}

impl OptionalCardCreator for Membership {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use wasm_bindgen_test::wasm_bindgen_test;

    // region Membership
    #[wasm_bindgen_test]
    fn should_create_card_for_known_membership() {
        let membership = Membership::new(
            "Doe".to_owned(),
            "Jon".to_owned(),
            "M".to_owned(),
            None,
            None,
            "123456".to_owned(),
            "email@address.org".to_owned(),
            true,
            Utc::now().date_naive(),
            false,
            "club".to_owned(),
            "structure_code".to_owned(),
        );
        let document = Document::new().unwrap();

        let element = Membership::create_card_from_optional(&Some(&membership), &document);
        let inner_html = element.inner_html();
        let expected_inner_html = "<div class=\"font-semibold\">Membre associé au numéro d'adhésion fourni</div><div>Nom : Doe</div><div>Prénom : Jon</div><div>Fin de l'adhésion : 05/03/2025</div><div>Adresse mail : email@address.org</div>";
        assert_eq!(expected_inner_html, inner_html);
    }

    #[wasm_bindgen_test]
    fn should_create_card_for_unknown_membership() {
        let document = Document::new().unwrap();

        let element = Membership::create_card_from_optional(&None, &document);
        let inner_html = element.inner_html();
        let expected_inner_html = "<div class=\"font-semibold\">Aucune adhésion trouvée</div>";
        assert_eq!(expected_inner_html, inner_html);
    }
    // endregion

    // region MemberToCheck
    #[wasm_bindgen_test]
    fn should_create_card_for_member_to_check() {
        let member_to_check = MemberToCheck::new("123456".to_owned(), "Doe".to_owned(), "Jon".to_owned());
        let document = Document::new().unwrap();

        let element = member_to_check.create_card(&document);
        let inner_html = element.inner_html();
        let expected_inner_html = "<div class=\"font-semibold\">Membre à vérifier</div><div>Numéro d'adhésion : 123456</div><div>Nom : Doe</div><div>Prénom : Jon</div>";
        assert_eq!(expected_inner_html, inner_html);
    }
    // endregion
}
