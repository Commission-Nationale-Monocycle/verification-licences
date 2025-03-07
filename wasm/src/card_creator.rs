use crate::utils::{
    ElementConfig, add_class, append_child, create_element, create_element_with_class,
    create_element_with_classes, create_element_with_options,
};
use chrono::Utc;
use dto::checked_member::CheckedMember;
use dto::member_to_check::MemberToCheck;
use dto::membership::Membership;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, Element};

pub trait CardCreator {
    fn create_card(&self, document: &Document) -> Element;
}

pub trait OptionalCardCreator {
    fn create_card_from_optional(element: &Option<&Self>, document: &Document) -> Element;
}

const MEMBERSHIP_CONTAINER_CLASS_NAME: &str = "membership-container";
const EMAIL_ADDRESS_CLASS_NAME: &str = "email-address-container";

impl OptionalCardCreator for Membership {
    fn create_card_from_optional(element: &Option<&Self>, document: &Document) -> Element {
        let container = create_element_with_classes(
            document,
            "div",
            None,
            None,
            &[
                MEMBERSHIP_CONTAINER_CLASS_NAME,
                "flex",
                "flex-col",
                "flex-shrink-0",
                "m-2",
            ],
        );
        if let Some(membership) = element {
            let name = format!("Nom : {}", membership.name());
            let firstname = format!("Prénom : {}", membership.firstname());
            let end_date = format!(
                "Fin de l'adhésion : {}",
                membership.end_date().format("%d/%m/%Y")
            );

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
            let email_container =
                create_element(document, "div", Some(&container), Some("Adresse mail : "));
            create_element_with_classes(
                document,
                "div",
                Some(&email_container),
                Some(&format!(
                    "<a href='mailto:{}'>{}</a>",
                    membership.email_address(),
                    membership.email_address()
                )),
                &[EMAIL_ADDRESS_CLASS_NAME, "inline"],
            );
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

        let member_to_check_card = self.member_to_check().create_card(document);
        append_child(&container, &member_to_check_card);

        let membership_card =
            Membership::create_card_from_optional(&self.membership().as_ref(), document);
        append_child(&container, &membership_card);

        {
            let membership_container = container
                .get_elements_by_class_name(MEMBERSHIP_CONTAINER_CLASS_NAME)
                .get_with_index(0)
                .expect_throw("can't find membership container");
            if let Some(membership) = &self.membership() {
                if Utc::now().date_naive() > *membership.end_date() {
                    add_class(&container, "bg-orange-300");

                    let checkbox = create_checkbox(document);
                    append_child(&membership_container, &checkbox);
                }
            } else {
                add_class(&container, "bg-red-300");
                add_class(&membership_container, "justify-center");
            }
        }

        container
    }
}

fn create_checkbox(document: &Document) -> Element {
    create_element_with_options(
        &document,
        "label",
        None,
        Some("Envoyer un email"),
        &ElementConfig::new(
            Some(&["border-2", "rounded-md", "flex", "justify-evenly"]),
            None,
            Some(&[&create_element_with_options(
                &document,
                "input",
                None,
                None,
                &ElementConfig::new(None, Some(&[("type", "checkbox"), ("checked", "")]), None),
            )]),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Days, Utc};
    use wasm_bindgen_test::wasm_bindgen_test;

    // region Membership
    #[wasm_bindgen_test]
    fn should_create_card_for_known_membership() {
        let end_date = Utc::now().date_naive();
        let membership = Membership::new(
            "Doe".to_owned(),
            "Jon".to_owned(),
            "M".to_owned(),
            None,
            None,
            "123456".to_owned(),
            "email@address.org".to_owned(),
            true,
            end_date,
            false,
            "club".to_owned(),
            "structure_code".to_owned(),
        );
        let document = Document::new().unwrap();

        let element = Membership::create_card_from_optional(&Some(&membership), &document);
        let inner_html = element.inner_html();
        let expected_inner_html = format!(
            "<div class=\"font-semibold\">Membre associé au numéro d'adhésion fourni</div><div>Nom : Doe</div><div>Prénom : Jon</div><div>Fin de l'adhésion : {}</div><div>Adresse mail : <div class=\"email-address-container inline\"><a href=\"mailto:email@address.org\">email@address.org</a></div></div>",
            end_date.format("%d/%m/%Y")
        );
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
        let member_to_check =
            MemberToCheck::new("123456".to_owned(), "Doe".to_owned(), "Jon".to_owned());
        let document = Document::new().unwrap();

        let element = member_to_check.create_card(&document);
        let inner_html = element.inner_html();
        let expected_inner_html = "<div class=\"font-semibold\">Membre à vérifier</div><div>Numéro d'adhésion : 123456</div><div>Nom : Doe</div><div>Prénom : Jon</div>";
        assert_eq!(expected_inner_html, inner_html);
    }
    // endregion

    // region CheckedMember
    #[wasm_bindgen_test]
    fn should_create_card_for_checked_member_with_up_to_date_membership() {
        let member_to_check =
            MemberToCheck::new("123456".to_owned(), "Doe".to_owned(), "Jon".to_owned());
        let end_date = Utc::now().date_naive();
        let membership = Membership::new(
            "Doe".to_owned(),
            "Jon".to_owned(),
            "M".to_owned(),
            None,
            None,
            "123456".to_owned(),
            "email@address.org".to_owned(),
            true,
            end_date,
            false,
            "club".to_owned(),
            "structure_code".to_owned(),
        );
        let checked_member = CheckedMember::new(member_to_check, Some(membership));
        let document = Document::new().unwrap();

        let element = checked_member.create_card(&document);
        let inner_html = element.inner_html();
        let expected_inner_html = format!(
            "<div class=\"flex-shrink-0 m-2\"><div class=\"font-semibold\">Membre à vérifier</div><div>Numéro d'adhésion : 123456</div><div>Nom : Doe</div><div>Prénom : Jon</div></div><div class=\"membership-container flex flex-col flex-shrink-0 m-2\"><div class=\"font-semibold\">Membre associé au numéro d'adhésion fourni</div><div>Nom : Doe</div><div>Prénom : Jon</div><div>Fin de l'adhésion : {}</div><div>Adresse mail : <div class=\"email-address-container inline\"><a href=\"mailto:email@address.org\">email@address.org</a></div></div></div>",
            end_date.format("%d/%m/%Y")
        );
        assert_eq!(expected_inner_html, inner_html);
    }

    #[wasm_bindgen_test]
    fn should_create_card_for_checked_member_with_expired_membership() {
        let member_to_check =
            MemberToCheck::new("123456".to_owned(), "Doe".to_owned(), "Jon".to_owned());
        let end_date = Utc::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap();
        let membership = Membership::new(
            "Doe".to_owned(),
            "Jon".to_owned(),
            "M".to_owned(),
            None,
            None,
            "123456".to_owned(),
            "email@address.org".to_owned(),
            true,
            end_date,
            true,
            "club".to_owned(),
            "structure_code".to_owned(),
        );
        let checked_member = CheckedMember::new(member_to_check, Some(membership));
        let document = Document::new().unwrap();

        let element = checked_member.create_card(&document);

        assert!(element.class_name().contains("bg-orange"));
        let inner_html = element.inner_html();
        let expected_inner_html = format!(
            "<div class=\"flex-shrink-0 m-2\"><div class=\"font-semibold\">Membre à vérifier</div><div>Numéro d'adhésion : 123456</div><div>Nom : Doe</div><div>Prénom : Jon</div></div><div class=\"membership-container flex flex-col flex-shrink-0 m-2\"><div class=\"font-semibold\">Membre associé au numéro d'adhésion fourni</div><div>Nom : Doe</div><div>Prénom : Jon</div><div>Fin de l'adhésion : {}</div><div>Adresse mail : <div class=\"email-address-container inline\"><a href=\"mailto:email@address.org\">email@address.org</a></div></div><label class=\"border-2 rounded-md flex justify-evenly\">Envoyer un email<input type=\"checkbox\" checked=\"\"/></label></div>",
            end_date.format("%d/%m/%Y")
        );
        assert_eq!(expected_inner_html, inner_html);
    }

    #[wasm_bindgen_test]
    fn should_create_card_for_checked_member_with_unknown_membership() {
        let member_to_check =
            MemberToCheck::new("123456".to_owned(), "Doe".to_owned(), "Jon".to_owned());
        let checked_member = CheckedMember::new(member_to_check, None);
        let document = Document::new().unwrap();

        let element = checked_member.create_card(&document);

        assert!(element.class_name().contains("bg-red"));
        let inner_html = element.inner_html();
        let expected_inner_html = "<div class=\"flex-shrink-0 m-2\"><div class=\"font-semibold\">Membre à vérifier</div><div>Numéro d'adhésion : 123456</div><div>Nom : Doe</div><div>Prénom : Jon</div></div><div class=\"membership-container flex flex-col flex-shrink-0 m-2 justify-center\"><div class=\"font-semibold\">Aucune adhésion trouvée</div></div>";
        assert_eq!(expected_inner_html, inner_html);
    }
    // endregion
}
