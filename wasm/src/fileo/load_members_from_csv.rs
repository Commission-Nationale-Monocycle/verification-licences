use crate::error::Error;
use crate::utils::get_element_by_id_dyn;
use csv::{Reader, StringRecord};
use dto::csv_member::CsvMember;
use log::warn;
use std::collections::BTreeSet;
use web_sys::{Document, HtmlInputElement, HtmlSelectElement};

enum MembersToCheckFileFormat {
    MembershipNumberLastNameFirstName,
    MembershipNumberIdentity,
    MembershipNumber,
    LastNameFirstName,
    Identity,
}

impl TryFrom<String> for MembersToCheckFileFormat {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "MembershipNumberLastNameFirstName" => {
                Ok(MembersToCheckFileFormat::MembershipNumberLastNameFirstName)
            }
            "MembershipNumberIdentity" => Ok(MembersToCheckFileFormat::MembershipNumberIdentity),
            "MembershipNumber" => Ok(MembersToCheckFileFormat::MembershipNumber),
            "LastNameFirstName" => Ok(MembersToCheckFileFormat::LastNameFirstName),
            "Identity" => Ok(MembersToCheckFileFormat::Identity),
            _ => Err(Error::new("Format inexistant", "Format doesn't exist")),
        }
    }
}

pub async fn load_members_to_check(
    document: &Document,
) -> crate::Result<(BTreeSet<CsvMember>, Vec<String>)> {
    let members_to_check_picker =
        get_element_by_id_dyn::<HtmlInputElement>(document, "members-to-check-picker")?;
    let members_to_check_format_selector =
        get_element_by_id_dyn::<HtmlSelectElement>(document, "members-to-check-format-selector")?;
    let format = members_to_check_format_selector.value();
    let format = MembersToCheckFileFormat::try_from(format)?;

    let csv_file = members_to_check_picker
        .files()
        .expect("no files")
        .get(0)
        .expect("file should be accessible");

    let promise = csv_file.text();
    let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let csv_content = text_jsvalue.as_string().ok_or_else(|| Error::new("Le fichier CSV contient des caractères incorrects. Vérifiez l'encodage UTF-8 du fichier.", "csv file should contain only valid UTF-8 characters"))?;

    let (members_to_check, wrong_lines) =
        load_members_to_check_from_csv_string(&csv_content, &format);
    Ok((members_to_check, wrong_lines))
}

/// Load members to check from a CSV-formatted String, such as:
/// `membership_num;name;firstname`
fn load_members_to_check_from_csv_string(
    members_to_check: &str,
    format: &MembersToCheckFileFormat,
) -> (BTreeSet<CsvMember>, Vec<String>) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(true)
        .from_reader(members_to_check.as_bytes());

    load_members_to_check_from_csv(&mut reader, format)
}

/// Load members to check rom a CSV-formatted Reader, such as:
/// `membership_num;name;firstname`
fn load_members_to_check_from_csv<T>(
    reader: &mut Reader<T>,
    format: &MembersToCheckFileFormat,
) -> (BTreeSet<CsvMember>, Vec<String>)
where
    T: std::io::Read,
{
    let mut members_to_check = BTreeSet::new();
    let mut wrong_lines = vec![];

    reader.records().for_each(|record| {
        match deserialize_member_to_check(record, format) {
            Ok(member) => {
                members_to_check.insert(member);
            }
            Err(wrong_line) => {
                if let Some(wrong_line) = wrong_line {
                    wrong_lines.push(wrong_line);
                }
            }
        };
    });

    (members_to_check, wrong_lines)
}

fn deserialize_member_to_check(
    record: Result<StringRecord, csv::Error>,
    format: &MembersToCheckFileFormat,
) -> Result<CsvMember, Option<String>> {
    if let Ok(record) = record {
        let fields_count = record.len();

        match format {
            MembersToCheckFileFormat::MembershipNumberLastNameFirstName => {
                if fields_count != 3 {
                    Err(Some(record.iter().collect::<Vec<_>>().join(";")))
                } else {
                    Ok(CsvMember::new(
                        Some(record.get(0).unwrap().to_owned()),
                        None,
                        Some(record.get(1).unwrap().to_owned()),
                        Some(record.get(2).unwrap().to_owned()),
                    ))
                }
            }
            MembersToCheckFileFormat::MembershipNumberIdentity => {
                if fields_count != 2 {
                    Err(Some(record.iter().collect::<Vec<_>>().join(";")))
                } else {
                    Ok(CsvMember::new(
                        Some(record.get(0).unwrap().to_owned()),
                        Some(record.get(1).unwrap().to_owned()),
                        None,
                        None,
                    ))
                }
            }
            MembersToCheckFileFormat::MembershipNumber => {
                if fields_count != 1 {
                    Err(Some(record.iter().collect::<Vec<_>>().join(";")))
                } else {
                    Ok(CsvMember::new(
                        Some(record.get(0).unwrap().to_owned()),
                        None,
                        None,
                        None,
                    ))
                }
            }
            MembersToCheckFileFormat::LastNameFirstName => {
                if fields_count != 2 {
                    Err(Some(record.iter().collect::<Vec<_>>().join(";")))
                } else {
                    Ok(CsvMember::new(
                        None,
                        None,
                        Some(record.get(0).unwrap().to_owned()),
                        Some(record.get(1).unwrap().to_owned()),
                    ))
                }
            }
            MembersToCheckFileFormat::Identity => {
                if fields_count != 1 {
                    Err(Some(record.iter().collect::<Vec<_>>().join(";")))
                } else {
                    Ok(CsvMember::new(
                        None,
                        Some(record.get(0).unwrap().to_owned()),
                        None,
                        None,
                    ))
                }
            }
        }
    } else if let Err(error) = record {
        warn!(
            "Error while deserializing member to check [error: {:?}]",
            error
        );
        Err(None)
    } else {
        Err(None)
    }
}

#[cfg(test)]
mod tests {
    mod load_members_to_check_from_csv_string {
        mod membership_num_last_name_first_name {
            use crate::fileo::load_members_from_csv::{
                MembersToCheckFileFormat, load_members_to_check_from_csv_string,
            };
            use MembersToCheckFileFormat::MembershipNumberLastNameFirstName;
            use dto::csv_member::CsvMember;
            use std::collections::BTreeSet;

            #[test]
            fn success() {
                let membership_num = "123".to_owned();
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{membership_num};{name};{first_name}");
                let result =
                    load_members_to_check_from_csv_string(&csv, &MembershipNumberLastNameFirstName);
                assert_eq!(
                    (
                        BTreeSet::from_iter(vec![CsvMember::new(
                            Some(membership_num),
                            None,
                            Some(name),
                            Some(first_name),
                        )]),
                        vec![]
                    ),
                    result
                )
            }

            #[test]
            fn fail_when_one_field() {
                let membership_num = "123".to_owned();
                let csv = membership_num.to_string();
                let result =
                    load_members_to_check_from_csv_string(&csv, &MembershipNumberLastNameFirstName);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }

            #[test]
            fn fail_when_two_fields() {
                let membership_num = "123".to_owned();
                let identity = "Doe John".to_owned();
                let csv = format!("{membership_num};{identity}");
                let result =
                    load_members_to_check_from_csv_string(&csv, &MembershipNumberLastNameFirstName);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }
        }

        mod membership_num_identity {
            use crate::fileo::load_members_from_csv::{
                MembersToCheckFileFormat, load_members_to_check_from_csv_string,
            };
            use MembersToCheckFileFormat::MembershipNumberIdentity;
            use dto::csv_member::CsvMember;
            use std::collections::BTreeSet;

            #[test]
            fn success() {
                let membership_num = "123".to_owned();
                let identity = "Doe John".to_owned();
                let csv = format!("{membership_num};{identity}");
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumberIdentity);
                assert_eq!(
                    (
                        BTreeSet::from_iter(vec![CsvMember::new(
                            Some(membership_num),
                            Some(identity),
                            None,
                            None,
                        )]),
                        vec![]
                    ),
                    result
                )
            }

            #[test]
            fn fail_when_one_field() {
                let membership_num = "123".to_owned();
                let csv = membership_num.to_string();
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumberIdentity);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }

            #[test]
            fn fail_when_three_fields() {
                let membership_num = "123".to_owned();
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{membership_num};{name};{first_name}");
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumberIdentity);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }
        }

        mod membership_num {
            use crate::fileo::load_members_from_csv::MembersToCheckFileFormat::MembershipNumber;
            use crate::fileo::load_members_from_csv::load_members_to_check_from_csv_string;
            use dto::csv_member::CsvMember;
            use std::collections::BTreeSet;

            #[test]
            fn success() {
                let membership_num = "123".to_owned();
                let csv = membership_num.to_string();
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumber);
                assert_eq!(
                    (
                        BTreeSet::from_iter(vec![CsvMember::new(
                            Some(membership_num),
                            None,
                            None,
                            None,
                        )]),
                        vec![]
                    ),
                    result
                )
            }

            #[test]
            fn fail_when_two_fields() {
                let membership_num = "123".to_owned();
                let identity = "Doe John".to_owned();
                let csv = format!("{membership_num};{identity}");
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumber);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }

            #[test]
            fn fail_when_three_fields() {
                let membership_num = "123".to_owned();
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{membership_num};{name};{first_name}");
                let result = load_members_to_check_from_csv_string(&csv, &MembershipNumber);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }
        }

        mod last_name_first_name {
            use crate::fileo::load_members_from_csv::{
                MembersToCheckFileFormat, load_members_to_check_from_csv_string,
            };
            use MembersToCheckFileFormat::LastNameFirstName;
            use dto::csv_member::CsvMember;
            use std::collections::BTreeSet;

            #[test]
            fn success() {
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{name};{first_name}");
                let result = load_members_to_check_from_csv_string(&csv, &LastNameFirstName);
                assert_eq!(
                    (
                        BTreeSet::from_iter(vec![CsvMember::new(
                            None,
                            None,
                            Some(name),
                            Some(first_name),
                        )]),
                        vec![]
                    ),
                    result
                )
            }

            #[test]
            fn fail_when_one_field() {
                let membership_num = "123".to_owned();
                let csv = membership_num.to_string();
                let result = load_members_to_check_from_csv_string(&csv, &LastNameFirstName);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }

            #[test]
            fn fail_when_three_fields() {
                let membership_num = "123".to_owned();
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{membership_num};{name};{first_name}");
                let result = load_members_to_check_from_csv_string(&csv, &LastNameFirstName);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }
        }

        mod identity {
            use crate::fileo::load_members_from_csv::{
                MembersToCheckFileFormat, load_members_to_check_from_csv_string,
            };
            use MembersToCheckFileFormat::Identity;
            use dto::csv_member::CsvMember;
            use std::collections::BTreeSet;

            #[test]
            fn success() {
                let identity = "Doe John".to_owned();
                let csv = identity.to_owned();
                let result = load_members_to_check_from_csv_string(&csv, &Identity);
                assert_eq!(
                    (
                        BTreeSet::from_iter(vec![
                            CsvMember::new(None, Some(identity), None, None,)
                        ]),
                        vec![]
                    ),
                    result
                )
            }

            #[test]
            fn fail_when_two_fields() {
                let membership_num = "123".to_owned();
                let identity = "Doe John".to_owned();
                let csv = format!("{membership_num};{identity}");
                let result = load_members_to_check_from_csv_string(&csv, &Identity);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }

            #[test]
            fn fail_when_three_fields() {
                let membership_num = "123".to_owned();
                let name = "Doe".to_owned();
                let first_name = "John".to_owned();
                let csv = format!("{membership_num};{name};{first_name}");
                let result = load_members_to_check_from_csv_string(&csv, &Identity);
                let expected_result = (BTreeSet::new(), vec![csv]);
                assert_eq!(expected_result, result)
            }
        }
    }
}
