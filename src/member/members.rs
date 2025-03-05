use csv::Reader;
use derive_getters::Getters;
use std::collections::{BTreeSet, HashMap};
use std::ops::Deref;
use crate::member::Membership;
use serde::{Deserialize, Serialize};

use crate::member::memberships::Memberships;
use crate::tools::log_message;

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
    pub fn check_members<'a>(
        &self,
        members_to_check: &'a [MemberToCheck],
    ) -> Vec<(&'a MemberToCheck, Option<&Membership>)> {
        members_to_check
            .iter()
            .map(|member_to_check| (member_to_check, self.check_member(member_to_check)))
            .collect()
    }

    fn check_member(&self, member_to_check: &MemberToCheck) -> Option<&Membership> {
        let membership_num_to_check = member_to_check.membership_num().clone();

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

#[derive(Debug, Getters, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MemberToCheck {
    membership_num: String,
    name: String,
    firstname: String,
}

impl MemberToCheck {
    /// Load members to check rom a CSV-formatted String, such as:
    /// `membership_num;name;firstname`
    /// Ignore malformed rows.
    pub fn load_members_to_check_from_csv_string(members_to_check: &str) -> BTreeSet<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .from_reader(members_to_check.as_bytes());

        Self::load_members_to_check_from_csv(&mut reader)
    }

    /// Load members to check rom a CSV-formatted Reader, such as:
    /// `membership_num;name;firstname`
    /// Ignore malformed rows.
    fn load_members_to_check_from_csv<T>(reader: &mut Reader<T>) -> BTreeSet<Self>
    where
        T: std::io::Read,
    {
        reader
            .deserialize()
            .filter_map(|result: Result<MemberToCheck, _>| match result {
                Ok(membership) => Some(membership),
                Err(e) => {
                    log_message("Error while reading members to check")(e);
                    None
                }
            })
            .collect::<BTreeSet<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};
    use dto::membership::tests::{get_expected_member, MEMBERSHIP_NUMBER, MEMBER_FIRSTNAME, MEMBER_NAME};
    use crate::member::members::{MemberToCheck, Members};
    use crate::member::memberships::Memberships;

    // region check_members
    #[test]
    fn members_should_be_checked() {
        let membership = get_expected_member();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership.clone()]),
        )]));
        let member_to_check = MemberToCheck {
            membership_num: MEMBERSHIP_NUMBER.to_owned(),
            name: MEMBER_NAME.to_owned(),
            firstname: MEMBER_FIRSTNAME.to_owned(),
        };

        assert_eq!(
            vec![(&member_to_check.clone(), Some(&membership))],
            members.check_members(&[member_to_check])
        );
    }

    #[test]
    fn members_should_not_be_checked() {
        let membership = get_expected_member();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership]),
        )]));
        let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = MemberToCheck {
            membership_num: invalid_membership_number,
            name: MEMBER_NAME.to_owned(),
            firstname: MEMBER_FIRSTNAME.to_owned(),
        };

        assert_eq!(
            vec![(&member_to_check.clone(), None)],
            members.check_members(&[member_to_check])
        );
    }
    // endregion

    // region check_member
    #[test]
    fn member_should_be_check() {
        let membership = get_expected_member();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership.clone()]),
        )]));
        let member_to_check = MemberToCheck {
            membership_num: MEMBERSHIP_NUMBER.to_owned(),
            name: MEMBER_NAME.to_owned(),
            firstname: MEMBER_FIRSTNAME.to_owned(),
        };

        assert_eq!(Some(&membership), members.check_member(&member_to_check));
    }

    #[test]
    fn member_should_not_be_check() {
        let membership = get_expected_member();
        let members = Members::from(HashMap::from([(
            MEMBERSHIP_NUMBER.to_string(),
            Memberships::from([membership]),
        )]));
        let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
        let member_to_check = MemberToCheck {
            membership_num: invalid_membership_number,
            name: MEMBER_NAME.to_owned(),
            firstname: MEMBER_FIRSTNAME.to_owned(),
        };

        assert_eq!(None, members.check_member(&member_to_check));
    }
    // endregion

    // region load_members_to_check_from_csv_string
    #[test]
    fn should_load_members_to_check_from_csv_string() {
        let membership_num = "123".to_owned();
        let name = "Doe".to_owned();
        let firstname = "John".to_owned();
        let csv = format!("{membership_num};{name};{firstname}");
        let result = MemberToCheck::load_members_to_check_from_csv_string(&csv);
        assert_eq!(
            BTreeSet::from_iter(vec![MemberToCheck {
                membership_num,
                name,
                firstname
            }]),
            result
        )
    }

    #[test]
    fn should_not_load_members_to_check_from_csv_string_when_wrong_row() {
        let membership_num = "123".to_owned();
        let name = "Doe".to_owned();
        let csv = format!("{membership_num};{name}");
        let result = MemberToCheck::load_members_to_check_from_csv_string(&csv);
        let expected_result = BTreeSet::new();
        assert_eq!(expected_result, result)
    }
    // endregion
}
