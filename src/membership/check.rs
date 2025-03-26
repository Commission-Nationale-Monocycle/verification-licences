use crate::membership::indexed_memberships::IndexedMemberships;
use crate::membership::memberships::Memberships;
use crate::tools::{normalize, normalize_opt};
use dto::checked_member::CheckResult::{Match, NoMatch, PartialMatch};
use dto::checked_member::{CheckResult, CheckedMember};
use dto::member_to_check::MemberToCheck;
use dto::membership::Membership;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

/// For each member, loops through each membership to check whether there is a match.
/// In case of multiple matches, the greater match is returned.
/// The greatness of a match is defined by its preciseness
/// (corresponding membership number, name, identity) and the matching membership's end date.
pub fn check_members<T: MemberToCheck>(
    grouped_memberships: &IndexedMemberships,
    members_to_check: Vec<T>,
) -> Vec<CheckedMember<T>> {
    members_to_check
        .into_iter()
        .map(|member_to_check| {
            CheckedMember::new(
                member_to_check.clone(),
                check_member(grouped_memberships, &member_to_check),
            )
        })
        .collect()
}

fn check_member<T: MemberToCheck>(
    indexed_memberships: &IndexedMemberships,
    member_to_check: &T,
) -> CheckResult {
    let mut matches = BTreeSet::new();

    let memberships_by_num = indexed_memberships.memberships_by_num();
    let memberships_by_name = indexed_memberships.memberships_by_name();
    let memberships_by_identity = indexed_memberships.memberships_by_identity();
    matches.extend(check_member_by_property(
        memberships_by_num,
        member_to_check,
        MemberToCheck::membership_num,
        check_member_name_or_identity,
    ));
    matches.extend(check_member_by_property(
        memberships_by_name,
        member_to_check,
        |member_to_check| {
            if member_to_check.last_name().is_some() && member_to_check.first_name().is_some() {
                Some((
                    normalize_opt(member_to_check.last_name()),
                    normalize_opt(member_to_check.first_name()),
                ))
            } else {
                None
            }
        },
        check_member_membership_num,
    ));
    matches.extend(check_member_by_property(
        memberships_by_identity,
        member_to_check,
        MemberToCheck::identity,
        check_member_name_or_identity,
    ));

    if let Some(check_result) = matches.last() {
        return check_result.clone();
    }

    NoMatch
}

/// If member given property is some, then apply the check function onto it for each membership.
/// Return every match and partial match.
fn check_member_by_property<K, T, F1, F2>(
    indexed_memberships: &HashMap<K, Memberships>,
    member_to_check: &T,
    property: F1,
    check: F2,
) -> BTreeSet<CheckResult>
where
    T: MemberToCheck,
    K: Hash + Eq,
    F1: Fn(&T) -> Option<K>,
    F2: Fn(&T, &Membership) -> CheckResult,
{
    let mut matches = BTreeSet::new();
    if let Some(key) = property(member_to_check) {
        if let Some(memberships) = indexed_memberships.get(&key) {
            for membership in memberships.iter() {
                let result = check(member_to_check, membership);
                if matches!(result, Match(_) | PartialMatch(_)) {
                    matches.insert(result);
                }
            }
        }
    }

    matches
}

fn check_member_name_or_identity<T: MemberToCheck>(
    member_to_check: &T,
    membership: &Membership,
) -> CheckResult {
    let last_name_to_check = member_to_check.last_name();
    let first_name_to_check = member_to_check.first_name();
    let identity_to_check = member_to_check.identity();

    let last_name = normalize(membership.name());
    let first_name = normalize(membership.first_name());

    if last_name_to_check.is_some() && first_name_to_check.is_some() {
        let last_name_to_check = normalize_opt(last_name_to_check);
        let first_name_to_check = normalize_opt(first_name_to_check);

        // All matches => that's a match!
        if last_name_to_check == last_name && first_name_to_check == first_name {
            return Match(membership.clone());
        }

        // Membership num matches, but name doesn't match => partial match /!\
        return PartialMatch(membership.clone());
    }

    if identity_to_check.is_some() {
        // In order for identity and names to match, we strip spaces.
        // This may lead to some strange behaviors ("Jon Doe" would be the same as "Jo Ndoe"),
        // but let's assume it's not an issue there.
        let identity_to_check = normalize_opt(identity_to_check);

        let last_name = last_name.split(" ").collect::<String>();
        let first_name = first_name.split(" ").collect::<String>();

        // If membership and identity match => that's a match!
        if identity_to_check == format!("{last_name}{first_name}")
            || identity_to_check == format!("{first_name}{last_name}")
        {
            return Match(membership.clone());
        }

        return PartialMatch(membership.clone());
    }

    PartialMatch(membership.clone())
}

fn check_member_membership_num<T: MemberToCheck>(
    member_to_check: &T,
    membership: &Membership,
) -> CheckResult {
    let membership_num_to_check = member_to_check.membership_num();
    let membership_num = normalize(membership.membership_number());
    if membership_num_to_check.is_some() {
        let membership_num_to_check = normalize_opt(membership_num_to_check);
        // Membership num is the primary identifier.
        // If it is provided but doesn't match, then there's no match.
        if membership_num_to_check != membership_num
            && membership_num_to_check.parse::<u32>().ok() != membership_num.parse::<u32>().ok()
        {
            return NoMatch;
        }

        return Match(membership.clone());
    }

    PartialMatch(membership.clone())
}

#[cfg(test)]
mod tests {
    mod check_members {
        use crate::membership::check::check_members;
        use crate::membership::indexed_memberships::IndexedMemberships;
        use dto::checked_member::CheckResult::{Match, NoMatch};
        use dto::checked_member::CheckedMember;
        use dto::csv_member::CsvMember;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };

        #[test]
        fn success() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership.clone()]);
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                vec![CheckedMember::new(
                    member_to_check.clone(),
                    Match(membership)
                )],
                check_members(&members, vec![member_to_check])
            );
        }

        #[test]
        fn fail() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership]);
            let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
            let member_to_check = CsvMember::new(
                invalid_membership_number,
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                vec![CheckedMember::new(member_to_check.clone(), NoMatch)],
                check_members(&members, vec![member_to_check])
            );
        }
    }

    mod check_member {
        use crate::membership::check::check_member;
        use crate::membership::indexed_memberships::IndexedMemberships;
        use chrono::Months;
        use dto::checked_member::CheckResult::{Match, NoMatch};
        use dto::csv_member::CsvMember;
        use dto::membership::Membership;
        use dto::membership::tests::{
            MEMBER_FIRST_NAME, MEMBER_NAME, MEMBERSHIP_NUMBER, get_expected_membership,
        };

        #[test]
        fn success() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership.clone()]);
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(Match(membership), check_member(&members, &member_to_check));
        }

        #[test]
        fn success_when_membership_number_prepended_with_0() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership.clone()]);
            let member_to_check = CsvMember::new(
                format!("0{MEMBERSHIP_NUMBER}"), // Prepending with a 0 should not change anything
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(Match(membership), check_member(&members, &member_to_check));
        }

        #[test]
        fn match_when_num_and_identity() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership.clone()]);
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(), // Prepending with a 0 should not change anything
                Some(format!("{} {}", MEMBER_NAME, MEMBER_FIRST_NAME)),
                None,
                None,
            );

            assert_eq!(Match(membership), check_member(&members, &member_to_check));
        }

        #[test]
        fn fail() {
            let membership = get_expected_membership();
            let members = IndexedMemberships::from(vec![membership]);
            let invalid_membership_number = format!("{MEMBERSHIP_NUMBER} oops");
            let member_to_check = CsvMember::new(
                invalid_membership_number,
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(NoMatch, check_member(&members, &member_to_check));
        }

        #[test]
        fn get_better_match() {
            let matching_membership = get_expected_membership();
            let partial_matching_membership = Membership::new(
                "Not the right name".to_owned(),
                "Not the right first name either".to_owned(),
                matching_membership.gender().to_owned(),
                matching_membership.birthdate().to_owned(),
                matching_membership.age().to_owned(),
                matching_membership.membership_number().to_owned(),
                matching_membership.email_address().to_owned(),
                matching_membership.payed().to_owned(),
                matching_membership.end_date().to_owned(),
                matching_membership.expired().to_owned(),
                matching_membership.club().to_owned(),
                matching_membership.structure_code().to_owned(),
            );
            let not_matching_membership = Membership::new(
                "Not the right name".to_owned(),
                "Not the right first name either".to_owned(),
                matching_membership.gender().to_owned(),
                matching_membership.birthdate().to_owned(),
                matching_membership.age().to_owned(),
                "Also wrong membership number".to_owned(),
                matching_membership.email_address().to_owned(),
                matching_membership.payed().to_owned(),
                matching_membership.end_date().to_owned(),
                matching_membership.expired().to_owned(),
                matching_membership.club().to_owned(),
                matching_membership.structure_code().to_owned(),
            );
            let members = IndexedMemberships::from(vec![
                matching_membership.clone(),
                partial_matching_membership,
                not_matching_membership,
            ]);
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                Match(matching_membership),
                check_member(&members, &member_to_check)
            );
        }

        #[test]
        fn get_better_match_when_different_end_dates() {
            let newest_membership = get_expected_membership();
            let oldest_membership = Membership::new(
                newest_membership.name().to_owned(),
                newest_membership.first_name().to_owned(),
                newest_membership.gender().to_owned(),
                newest_membership.birthdate().to_owned(),
                newest_membership.age().to_owned(),
                newest_membership.membership_number().to_owned(),
                newest_membership.email_address().to_owned(),
                newest_membership.payed().to_owned(),
                newest_membership
                    .end_date()
                    .to_owned()
                    .checked_sub_months(Months::new(12))
                    .unwrap(),
                newest_membership.expired().to_owned(),
                newest_membership.club().to_owned(),
                newest_membership.structure_code().to_owned(),
            );
            let members =
                IndexedMemberships::from(vec![newest_membership.clone(), oldest_membership]);
            let member_to_check = CsvMember::new(
                MEMBERSHIP_NUMBER.to_owned(),
                None,
                Some(MEMBER_NAME.to_owned()),
                Some(MEMBER_FIRST_NAME.to_owned()),
            );

            assert_eq!(
                Match(newest_membership),
                check_member(&members, &member_to_check)
            );
        }
    }

    mod check_member_name_or_identity {
        use crate::membership::check::check_member_name_or_identity;
        use chrono::NaiveDate;
        use dto::checked_member::CheckResult;
        use dto::csv_member::CsvMember;
        use dto::membership::Membership;
        use dto::membership::tests::{MEMBERSHIP_NUMBER, get_expected_membership};

        #[test]
        fn name_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                None,
                Some(membership.name().to_owned()),
                Some(membership.first_name().to_owned()),
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::Match(membership), result);
        }

        #[test]
        fn normalized_name_match() {
            let membership = Membership::new(
                "Doe".to_string(),
                "Jon".to_string(),
                "H".to_string(),
                NaiveDate::from_ymd_opt(1980, 2, 1),
                Some(45),
                MEMBERSHIP_NUMBER.to_string(),
                "email@address.com".to_string(),
                true,
                NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
                false,
                "My club".to_string(),
                "Z01234".to_string(),
            );
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                None,
                Some("Do-é".to_owned()),
                Some(membership.first_name().to_owned()),
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::Match(membership), result);
        }

        #[test]
        fn last_name_doesnt_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                None,
                Some(format!(
                    "A whole other name: {}",
                    membership.name().to_owned()
                )),
                Some(membership.first_name().to_owned()),
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }

        #[test]
        fn first_name_doesnt_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                None,
                Some(membership.name().to_owned()),
                Some(format!(
                    "A whole other first name: {}",
                    membership.first_name().to_owned()
                )),
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }

        #[test]
        fn last_name_and_first_name_dont_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                None,
                Some(format!(
                    "A whole other name: {}",
                    membership.name().to_owned()
                )),
                Some(format!(
                    "A whole other first name: {}",
                    membership.first_name().to_owned()
                )),
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }

        #[test]
        fn identity_in_order_first_last_name_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                Some(format!("{} {}", membership.first_name(), membership.name())),
                None,
                None,
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::Match(membership), result);
        }

        #[test]
        fn identity_in_order_last_first_name_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                Some(format!("{} {}", membership.name(), membership.first_name())),
                None,
                None,
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::Match(membership), result);
        }

        #[test]
        fn identity_doesnt_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                membership.membership_number().to_owned(),
                Some(format!(
                    "{} {} Oops",
                    membership.name(),
                    membership.first_name()
                )),
                None,
                None,
            );

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }

        #[test]
        fn no_name_no_identity() {
            let membership = get_expected_membership();
            let member_to_check =
                CsvMember::new(membership.membership_number().to_owned(), None, None, None);

            let result = check_member_name_or_identity(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }
    }

    mod check_member_membership_num {
        use crate::membership::check::check_member_membership_num;
        use dto::checked_member::CheckResult;
        use dto::csv_member::CsvMember;
        use dto::membership::tests::get_expected_membership;
        use dto::uda_member::UdaMember;

        #[test]
        fn num_doesnt_match() {
            let membership = get_expected_membership();
            let member_to_check = CsvMember::new(
                format!("{} oops", membership.membership_number().to_owned()),
                None,
                Some(membership.name().to_owned()),
                Some(membership.first_name().to_owned()),
            );

            let result = check_member_membership_num(&member_to_check, &membership);
            assert_eq!(CheckResult::NoMatch, result);
        }

        #[test]
        fn no_num() {
            let membership = get_expected_membership();
            let member_to_check = UdaMember::new(
                1,
                None,
                membership.first_name().to_owned(),
                membership.name().to_owned(),
                "address@email.org".to_owned(),
                None,
                false,
            );

            let result = check_member_membership_num(&member_to_check, &membership);
            assert_eq!(CheckResult::PartialMatch(membership), result);
        }
    }
}
