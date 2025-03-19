use crate::member::Membership;
use crate::member::memberships::Memberships;
use dto::checked_member::CheckedMember;
use dto::member_identifier::MemberIdentifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

/// A map of [Memberships], grouped by membership number.
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Members {
    members: HashMap<String, Memberships>,
}

impl Deref for Members {
    type Target = HashMap<String, Memberships>;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl From<HashMap<String, Memberships>> for Members {
    fn from(members: HashMap<String, Memberships>) -> Self {
        Members { members }
    }
}

impl Members {
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
                if *known_membership_num == membership_num_to_check {
                    known_memberships_for_num.find_last_membership()
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::member::members::Members;
    use crate::member::memberships::Memberships;
    use dto::checked_member::CheckedMember;
    use dto::member_to_check::MemberToCheck;
    use dto::membership::tests::{
        MEMBER_FIRSTNAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
    };
    use std::collections::HashMap;

    // region check_members
    #[test]
    fn members_should_be_checked() {
        let membership = get_expected_membership();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership.clone()]),
        )]));
        let member_to_check = MemberToCheck::new(
            MEMBERSHIP_NUMBER.to_owned(),
            MEMBER_NAME.to_owned(),
            MEMBER_FIRSTNAME.to_owned(),
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
    fn members_should_not_be_checked() {
        let membership = get_expected_membership();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership]),
        )]));
        let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = MemberToCheck::new(
            invalid_membership_number,
            MEMBER_NAME.to_owned(),
            MEMBER_FIRSTNAME.to_owned(),
        );

        assert_eq!(
            vec![CheckedMember::new(member_to_check.clone(), None)],
            members.check_members(vec![member_to_check])
        );
    }
    // endregion

    // region check_member
    #[test]
    fn member_should_be_checked() {
        let membership = get_expected_membership();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership.clone()]),
        )]));
        let member_to_check = MemberToCheck::new(
            MEMBERSHIP_NUMBER.to_owned(),
            MEMBER_NAME.to_owned(),
            MEMBER_FIRSTNAME.to_owned(),
        );

        assert_eq!(Some(&membership), members.check_member(&member_to_check));
    }

    #[test]
    fn member_should_not_be_checked() {
        let membership = get_expected_membership();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership]),
        )]));
        let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = MemberToCheck::new(
            invalid_membership_number,
            MEMBER_NAME.to_owned(),
            MEMBER_FIRSTNAME.to_owned(),
        );

        assert_eq!(None, members.check_member(&member_to_check));
    }
    // endregion
}
