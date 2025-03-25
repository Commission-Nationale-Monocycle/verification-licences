use crate::membership::grouped_memberships::GroupedMemberships;
use dto::checked_member::CheckedMember;
use dto::member_identifier::MemberIdentifier;
use dto::membership::Membership;

/// Maps each member to check to its corresponding member if known.
/// Otherwise, maps it to [None].
pub fn check_members<T>(
    grouped_memberships: &GroupedMemberships,
    members_to_check: Vec<T>,
) -> Vec<CheckedMember<T>>
where
    T: MemberIdentifier,
{
    members_to_check
        .into_iter()
        .map(|member_to_check| {
            CheckedMember::new(
                member_to_check.clone(),
                check_member(grouped_memberships, &member_to_check).cloned(),
            )
        })
        .collect()
}

fn check_member<'a, T>(
    grouped_memberships: &'a GroupedMemberships,
    member_to_check: &T,
) -> Option<&'a Membership>
where
    T: MemberIdentifier,
{
    let membership_num_to_check = member_to_check.membership_num().clone()?;

    grouped_memberships
        .iter()
        .find_map(|(known_membership_num, known_memberships_for_num)| {
            if *known_membership_num == membership_num_to_check
                || known_membership_num.parse::<u32>() == membership_num_to_check.parse::<u32>()
            // Accounting for membership numbers starting with a 0 that could have been stripped by LibreOffice Calc or Excel
            {
                known_memberships_for_num.find_last_membership()
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    mod check_members {
        use crate::membership::check::check_members;
        use crate::membership::grouped_memberships::GroupedMemberships;
        use crate::membership::memberships::Memberships;
        use dto::checked_member::CheckedMember;
        use dto::csv_member::CsvMember;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };
        use std::collections::HashMap;

        #[test]
        fn success() {
            let membership = get_expected_membership();
            let members = GroupedMemberships::from(HashMap::from([(
                MEMBERSHIP_NUMBER.to_string(),
                Memberships::from([membership.clone()]),
            )]));
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                vec![CheckedMember::new(
                    member_to_check.clone(),
                    Some(membership)
                )],
                check_members(&members, vec![member_to_check])
            );
        }

        #[test]
        fn fail() {
            let membership = get_expected_membership();
            let members = GroupedMemberships::from(HashMap::from([(
                MEMBERSHIP_NUMBER.to_string(),
                Memberships::from([membership]),
            )]));
            let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
            let member_to_check = CsvMember::new(
                invalid_membership_number,
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                vec![CheckedMember::new(member_to_check.clone(), None)],
                check_members(&members, vec![member_to_check])
            );
        }
    }

    mod check_member {
        use crate::membership::check::check_member;
        use crate::membership::grouped_memberships::GroupedMemberships;
        use crate::membership::memberships::Memberships;
        use dto::csv_member::CsvMember;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };
        use std::collections::HashMap;

        #[test]
        fn success() {
            let membership = get_expected_membership();
            let members = GroupedMemberships::from(HashMap::from([(
                MEMBERSHIP_NUMBER.to_string(),
                Memberships::from([membership.clone()]),
            )]));
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(Some(&membership), check_member(&members, &member_to_check));
        }

        #[test]
        fn success_when_membership_number_prepended_with_0() {
            let membership = get_expected_membership();
            let members = GroupedMemberships::from(HashMap::from([(
                MEMBERSHIP_NUMBER.to_string(),
                Memberships::from([membership.clone()]),
            )]));
            let member_to_check = CsvMember::new(
                format!("0{MEMBERSHIP_NUMBER}"), // Prepending with a 0 should not change anything
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(Some(&membership), check_member(&members, &member_to_check));
        }

        #[test]
        fn fail() {
            let membership = get_expected_membership();
            let members = GroupedMemberships::from(HashMap::from([(
                MEMBERSHIP_NUMBER.to_string(),
                Memberships::from([membership]),
            )]));
            let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
            let member_to_check = CsvMember::new(
                invalid_membership_number,
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(None, check_member(&members, &member_to_check));
        }
    }
}
