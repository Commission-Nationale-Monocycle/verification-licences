use crate::error::Error;
use csv::{Reader, StringRecord};
use dto::csv_member::CsvMember;
use log::warn;
use std::collections::BTreeSet;
use web_sys::HtmlInputElement;

pub async fn load_members_to_check(
    members_to_check_picker: &HtmlInputElement,
) -> crate::Result<(BTreeSet<CsvMember>, Vec<String>)> {
    let csv_file = members_to_check_picker
        .files()
        .expect("no files")
        .get(0)
        .expect("file should be accessible");

    let promise = csv_file.text();
    let text_jsvalue = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let csv_content = text_jsvalue.as_string().ok_or_else(|| Error::new("Le fichier CSV contient des caractères incorrects. Vérifiez l'encodage UTF-8 du fichier.", "csv file should contain only valid UTF-8 characters"))?;

    let (members_to_check, wrong_lines) = load_members_to_check_from_csv_string(&csv_content);
    Ok((members_to_check, wrong_lines))
}

/// Load members to check from a CSV-formatted String, such as:
/// `membership_num;name;firstname`
pub fn load_members_to_check_from_csv_string(
    members_to_check: &str,
) -> (BTreeSet<CsvMember>, Vec<String>) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(true)
        .from_reader(members_to_check.as_bytes());

    load_members_to_check_from_csv(&mut reader)
}

/// Load members to check rom a CSV-formatted Reader, such as:
/// `membership_num;name;firstname`
fn load_members_to_check_from_csv<T>(reader: &mut Reader<T>) -> (BTreeSet<CsvMember>, Vec<String>)
where
    T: std::io::Read,
{
    let mut members_to_check = BTreeSet::new();
    let mut wrong_lines = vec![];

    reader.records().for_each(|record| {
        match deserialize_member_to_check(record) {
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
    record: std::result::Result<StringRecord, csv::Error>,
) -> std::result::Result<CsvMember, Option<String>> {
    if let Ok(record) = record {
        let fields_count = record.len();
        if fields_count == 2 {
            Ok(CsvMember::new(
                record.get(0).unwrap().to_owned(),
                Some(record.get(1).unwrap().to_owned()),
                None,
                None,
            ))
        } else if fields_count == 3 {
            Ok(CsvMember::new(
                record.get(0).unwrap().to_owned(),
                None,
                Some(record.get(1).unwrap().to_owned()),
                Some(record.get(2).unwrap().to_owned()),
            ))
        } else {
            Err(Some(record.iter().collect::<Vec<_>>().join(";")))
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
        use crate::fileo::load_members_from_csv::load_members_to_check_from_csv_string;
        use dto::csv_member::CsvMember;
        use std::collections::BTreeSet;

        #[test]
        fn success_with_name_and_firstname() {
            let membership_num = "123".to_owned();
            let name = "Doe".to_owned();
            let first_name = "John".to_owned();
            let csv = format!("{membership_num};{name};{first_name}");
            let result = load_members_to_check_from_csv_string(&csv);
            assert_eq!(
                (
                    BTreeSet::from_iter(vec![CsvMember::new(
                        membership_num,
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
        fn success_with_identity() {
            let membership_num = "123".to_owned();
            let identity = "Doe John".to_owned();
            let csv = format!("{membership_num};{identity}");
            let result = load_members_to_check_from_csv_string(&csv);
            assert_eq!(
                (
                    BTreeSet::from_iter(vec![CsvMember::new(
                        membership_num,
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
        fn fail_when_wrong_row() {
            let membership_num = "123".to_owned();
            let csv = membership_num.to_string();
            let result = load_members_to_check_from_csv_string(&csv);
            let expected_result = (BTreeSet::new(), vec![csv]);
            assert_eq!(expected_result, result)
        }
    }
}
