use crate::membership::indexed_memberships::IndexedMemberships;
use crate::tools::normalize;
use dto::member_to_look_up::MemberToLookUp;
use dto::membership::Membership;

pub fn look_member_up<'a>(
    indexed_memberships: &'a IndexedMemberships,
    member_to_look_up: &MemberToLookUp,
) -> Vec<&'a Membership> {
    look_member_up_by_membership_num(indexed_memberships, member_to_look_up)
        .or_else(|| look_member_up_by_last_name(indexed_memberships, member_to_look_up))
        .or_else(|| look_member_up_by_first_name(indexed_memberships, member_to_look_up))
        .unwrap_or_default()
}

fn look_member_up_by_membership_num<'a>(
    indexed_memberships: &'a IndexedMemberships,
    member_to_look_up: &MemberToLookUp,
) -> Option<Vec<&'a Membership>> {
    if let Some(membership_num) = member_to_look_up.membership_num() {
        let memberships = indexed_memberships
            .memberships_by_num()
            .get(&normalize(membership_num));
        if let Some(memberships) = memberships {
            let result = memberships
                .iter()
                .filter(|membership| {
                    if let Some(last_name) = member_to_look_up.last_name() {
                        if normalize(last_name) != normalize(membership.name()) {
                            return false;
                        }
                    }

                    if let Some(first_name) = member_to_look_up.first_name() {
                        if normalize(first_name) != normalize(membership.first_name()) {
                            return false;
                        }
                    }

                    true
                })
                .collect();
            Some(result)
        } else {
            Some(Vec::new())
        }
    } else {
        None
    }
}

fn look_member_up_by_last_name<'a>(
    indexed_memberships: &'a IndexedMemberships,
    member_to_look_up: &MemberToLookUp,
) -> Option<Vec<&'a Membership>> {
    if let Some(last_name) = member_to_look_up.last_name() {
        let memberships = indexed_memberships
            .memberships_by_last_name()
            .get(&normalize(last_name));
        if let Some(memberships) = memberships {
            let result = memberships
                .iter()
                .filter(|membership| {
                    if let Some(first_name) = member_to_look_up.first_name() {
                        if normalize(first_name) != normalize(membership.first_name()) {
                            return false;
                        }
                    }

                    true
                })
                .collect();
            Some(result)
        } else {
            Some(Vec::new())
        }
    } else {
        None
    }
}

fn look_member_up_by_first_name<'a>(
    indexed_memberships: &'a IndexedMemberships,
    member_to_look_up: &MemberToLookUp,
) -> Option<Vec<&'a Membership>> {
    if let Some(first_name) = member_to_look_up.first_name() {
        let memberships = indexed_memberships
            .memberships_by_first_name()
            .get(&normalize(first_name));
        let result = memberships
            .map(|memberships| memberships.iter().collect())
            .unwrap_or_default();
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    mod look_member_up {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use crate::membership::look_up::look_member_up;
        use dto::member_to_look_up::MemberToLookUp;
        use dto::membership::Membership;

        #[test]
        fn by_membership_num() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up =
                MemberToLookUp::new(Some(jon_doe().membership_number().to_owned()), None, None);

            let result = look_member_up(&memberships, &member_to_look_up);

            assert_eq!(vec![&jon_doe_previous_membership(), &jon_doe()], result);
        }

        #[test]
        fn by_last_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up =
                MemberToLookUp::new(None, Some(jon_doe().name().to_owned()), None);

            let result = look_member_up(&memberships, &member_to_look_up);

            assert_eq!(
                vec![&jon_doe_previous_membership(), &jon_doe(), &other_jon_doe()],
                result
            );
        }

        #[test]
        fn by_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up =
                MemberToLookUp::new(None, None, Some(jon_doe().first_name().to_owned()));

            let result = look_member_up(&memberships, &member_to_look_up);

            assert_eq!(
                vec![&jon_doe_previous_membership(), &jon_doe(), &other_jon_doe()],
                result
            );
        }

        #[test]
        fn no_criteria() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(None, None, None);

            let result = look_member_up(&memberships, &member_to_look_up);

            assert_eq!(Vec::<&Membership>::new(), result);
        }
    }

    mod look_member_up_by_membership_num {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use crate::membership::look_up::look_member_up_by_membership_num;
        use dto::member_to_look_up::MemberToLookUp;
        use dto::membership::Membership;

        #[test]
        fn no_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up =
                MemberToLookUp::new(Some(jon_doe().membership_number().to_owned()), None, None);

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(vec![&jon_doe_previous_membership(), &jon_doe()], result);
        }

        #[test]
        fn last_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                None,
            );

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(vec![&jon_doe_previous_membership(), &jon_doe()], result);
        }

        #[test]
        fn first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                None,
                Some(jon_doe().first_name().to_owned()),
            );

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(vec![&jon_doe_previous_membership(), &jon_doe()], result);
        }

        #[test]
        fn last_name_and_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                Some(jon_doe().first_name().to_owned()),
            );

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(vec![&jon_doe_previous_membership(), &jon_doe()], result);
        }

        #[test]
        fn no_num() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(None, None, None);

            let result = look_member_up_by_membership_num(&memberships, &member_to_look_up);

            assert_eq!(None, result);
        }

        #[test]
        fn no_matching_num() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(Some("123456789".to_owned()), None, None);

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(Vec::<&Membership>::new(), result);
        }

        #[test]
        fn no_matching_last_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some("Simpson".to_owned()),
                None,
            );

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(Vec::<&Membership>::new(), result);
        }

        #[test]
        fn matching_last_name_but_not_matching_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                Some("Homer".to_owned()),
            );

            let result =
                look_member_up_by_membership_num(&memberships, &member_to_look_up).unwrap();

            assert_eq!(Vec::<&Membership>::new(), result);
        }
    }

    mod look_member_up_by_last_name {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use crate::membership::look_up::look_member_up_by_last_name;
        use dto::member_to_look_up::MemberToLookUp;
        use dto::membership::Membership;

        #[test]
        fn no_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                None,
            );

            let result = look_member_up_by_last_name(&memberships, &member_to_look_up).unwrap();

            assert_eq!(
                vec![&jon_doe_previous_membership(), &jon_doe(), &other_jon_doe()],
                result
            );
        }

        #[test]
        fn first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                Some(jon_doe().first_name().to_owned()),
            );

            let result = look_member_up_by_last_name(&memberships, &member_to_look_up).unwrap();

            assert_eq!(
                vec![&jon_doe_previous_membership(), &jon_doe(), &other_jon_doe()],
                result
            );
        }

        #[test]
        fn no_last_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up =
                MemberToLookUp::new(Some(jon_doe().membership_number().to_owned()), None, None);

            let result = look_member_up_by_last_name(&memberships, &member_to_look_up);

            assert_eq!(None, result);
        }

        #[test]
        fn no_matching_last_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some("Simpson".to_owned()),
                None,
            );

            let result = look_member_up_by_last_name(&memberships, &member_to_look_up).unwrap();

            assert_eq!(Vec::<&Membership>::new(), result);
        }

        #[test]
        fn no_matching_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                Some("Homer".to_owned()),
            );

            let result = look_member_up_by_last_name(&memberships, &member_to_look_up).unwrap();

            assert_eq!(Vec::<&Membership>::new(), result);
        }
    }

    mod look_member_up_by_first_name {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use crate::membership::look_up::look_member_up_by_first_name;
        use dto::member_to_look_up::MemberToLookUp;

        #[test]
        fn no_first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                None,
            );

            let result = look_member_up_by_first_name(&memberships, &member_to_look_up);

            assert_eq!(None, result);
        }

        #[test]
        fn first_name() {
            let memberships = IndexedMemberships::from(vec![
                jonette_snow(),
                jon_doe(),
                jon_doe_previous_membership(),
                other_jon_doe(),
            ]);
            let member_to_look_up = MemberToLookUp::new(
                Some(jon_doe().membership_number().to_owned()),
                Some(jon_doe().name().to_owned()),
                Some(jon_doe().first_name().to_owned()),
            );

            let result = look_member_up_by_first_name(&memberships, &member_to_look_up).unwrap();

            assert_eq!(
                vec![&jon_doe_previous_membership(), &jon_doe(), &other_jon_doe()],
                result
            );
        }
    }
}
