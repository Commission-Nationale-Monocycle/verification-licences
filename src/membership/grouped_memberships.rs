use crate::membership::memberships::Memberships;
use dto::checked_member::CheckedMember;
use dto::member_identifier::MemberIdentifier;
use dto::membership::Membership;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

/// A map of [Memberships], grouped by membership number.
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct GroupedMemberships {
    memberships: HashMap<String, Memberships>,
}

impl Deref for GroupedMemberships {
    type Target = HashMap<String, Memberships>;

    fn deref(&self) -> &Self::Target {
        &self.memberships
    }
}

impl From<HashMap<String, Memberships>> for GroupedMemberships {
    fn from(members: HashMap<String, Memberships>) -> Self {
        GroupedMemberships {
            memberships: members,
        }
    }
}

impl GroupedMemberships {
    /// Maps each member to check to its corresponding member if known.
    /// Otherwise, maps it to [None].
    pub fn check_members<T>(&self, members_to_check: Vec<T>) -> Vec<CheckedMember<T>>
    where
        T: MemberIdentifier,
    {
        members_to_check
            .into_iter()
            .map(|member_to_check| {
                CheckedMember::new(
                    member_to_check.clone(),
                    self.check_member(&member_to_check).cloned(),
                )
            })
            .collect()
    }

    fn check_member<T>(&self, member_to_check: &T) -> Option<&Membership>
    where
        T: MemberIdentifier,
    {
        let membership_num_to_check = member_to_check.membership_num().clone()?;

        self.iter()
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
}

#[cfg(test)]
mod tests {
    mod check_members {
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
                members.check_members(vec![member_to_check])
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
                members.check_members(vec![member_to_check])
            );
        }
    }

    mod check_member {
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

            assert_eq!(Some(&membership), members.check_member(&member_to_check));
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

            assert_eq!(Some(&membership), members.check_member(&member_to_check));
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

            assert_eq!(None, members.check_member(&member_to_check));
        }
    }
}
