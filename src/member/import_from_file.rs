use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::str::FromStr;

use chrono::NaiveDate;
use csv::Reader;
use regex::bytes::{Captures, Regex};

use crate::error::Result;
use crate::fileo::imported_membership::ImportedMembership;
use crate::member::Membership;
use crate::member::error::MembershipError::{
    CantBrowseThroughFiles, CantConvertDateFieldToString, CantOpenMembersFile,
    CantOpenMembersFileFolder, InvalidDate, NoFileFound, WrongRegex,
};
use crate::member::file_details::FileDetails;
use crate::member::members::Members;
use crate::member::memberships::Memberships;
use crate::tools::{log_message, log_message_and_return};

/// Load a list of [Member]s from a file containing [Membership]s.
/// Members are memberships grouped by membership num.
pub fn import_from_file(filepath: &OsStr) -> Result<Members> {
    let error_message = format!("Can't open members file `{:?}`.", filepath.to_str());
    let file = File::open(filepath).map_err(log_message_and_return(
        &error_message,
        CantOpenMembersFile(filepath.to_owned()),
    ))?;
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let members = load_memberships(&mut reader);
    Ok(group_members_by_membership(members))
}

fn load_memberships<T>(reader: &mut Reader<T>) -> Vec<Membership>
where
    T: std::io::Read,
{
    reader
        .deserialize()
        .filter_map(|result: Result<ImportedMembership, _>| match result {
            Ok(membership) => Some(membership.into()),
            Err(e) => {
                log_message("Error while reading membership")(e);
                None
            }
        })
        .collect::<Vec<_>>()
}

fn group_members_by_membership(memberships: Vec<Membership>) -> Members {
    let mut map = HashMap::new();

    memberships.into_iter().for_each(|membership| {
        let membership_number = membership.membership_number().to_string();
        map.entry(membership_number)
            .and_modify(|memberships: &mut Memberships| {
                memberships.insert(membership.clone());
            })
            .or_insert(Memberships::from([membership.clone(); 1]));
    });

    Members::from(map)
}

pub fn find_file(members_file_folder: &OsStr) -> Result<FileDetails> {
    check_folder(members_file_folder)?;

    let regex = "^memberships-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$";
    let regex = Regex::new(regex).or(Err(WrongRegex(regex.to_owned())))?;
    let paths = fs::read_dir(members_file_folder)
        .or(Err(CantBrowseThroughFiles(members_file_folder.to_owned())))?;
    for path in paths {
        let path = path.expect("Path should be valid.");
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if let Some(captures) = captures {
            let date = convert_captures_to_date(&captures)?;
            return Ok(FileDetails::new(date, path.path().into_os_string()));
        }
    }

    Err(NoFileFound)?
}

fn check_folder(members_file_folder: &OsStr) -> Result<()> {
    match fs::exists(members_file_folder) {
        Ok(true) => Ok(()),
        Ok(false) => Err(NoFileFound)?,
        Err(e) => {
            log_message(&format!(
                "`{members_file_folder:?}` folder is inaccessible."
            ))(e);
            Err(CantOpenMembersFileFolder(members_file_folder.to_owned()))?
        }
    }
}

fn convert_captures_to_date(captures: &Captures) -> Result<NaiveDate> {
    let date = NaiveDate::from_ymd_opt(
        convert_match_to_integer(captures, "year")?,
        convert_match_to_integer(captures, "month")?,
        convert_match_to_integer(captures, "day")?,
    )
    .ok_or(InvalidDate)?;
    Ok(date)
}

fn convert_match_to_integer<T: FromStr>(captures: &Captures, key: &str) -> Result<T> {
    let result = String::from_utf8_lossy(&captures[key])
        .parse::<T>()
        .or(Err(CantConvertDateFieldToString))?;
    Ok(result)
}

/// Clean all files in folder that aren't given file.
pub fn clean_old_files(members_file_folder: &OsStr, file_update_date: &NaiveDate) -> Result<()> {
    let regex = build_members_file_regex()?;
    let paths = fs::read_dir(members_file_folder)
        .or(Err(CantBrowseThroughFiles(members_file_folder.to_owned())))?;
    for path in paths {
        let path = path.expect("Path should be valid.");
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if let Some(captures) = captures {
            let date = convert_captures_to_date(&captures)?;
            if &date != file_update_date {
                fs::remove_file(path.path()).ok();
            }
        }
    }

    Ok(())
}

fn build_members_file_regex() -> Result<Regex> {
    let regex = "^memberships-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$";
    let regex = Regex::new(regex).or(Err(WrongRegex(regex.to_owned())))?;
    Ok(regex)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;

    use crate::error::ApplicationError;
    use crate::member::Membership;
    use crate::member::error::MembershipError::{
        CantConvertDateFieldToString, CantOpenMembersFile, InvalidDate, NoFileFound,
    };
    use crate::member::import_from_file::{
        build_members_file_regex, check_folder, convert_captures_to_date, convert_match_to_integer,
        find_file, group_members_by_membership, import_from_file, load_memberships,
    };
    use crate::member::members::Members;
    use crate::member::memberships::Memberships;
    use crate::tools::test::tests::temp_dir;
    use chrono::NaiveDate;
    use dto::membership::tests::{
        get_expected_membership, get_malformed_member_as_csv, get_member_as_csv,
    };
    use regex::bytes::Regex;

    // region import_from_file
    #[test]
    fn should_import_from_file() {
        let dir = temp_dir();
        let file_name = "memberships.csv";
        let file_path = dir.join(file_name);

        fs::write(&file_path, get_member_as_csv()).unwrap();

        let result = import_from_file(file_path.as_ref()).unwrap();
        assert_eq!(
            &Memberships::from([get_expected_membership()]),
            result.get("123456").unwrap()
        )
    }

    #[test]
    fn should_not_import_from_file_when_cant_open_file() {
        let dir = temp_dir();
        let file_name = "memberships.csv";
        let file_path = dir.join(file_name);

        let result = import_from_file(file_path.as_ref()).err().unwrap();
        assert!(matches!(
            result,
            ApplicationError::Membership(CantOpenMembersFile(_))
        ));
    }

    // endregion

    // region load_members
    #[test]
    fn should_load_members() {
        let entry = get_member_as_csv();
        let expected_member = get_expected_membership();

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(entry.as_bytes()));
        let members = load_memberships(&mut reader);
        assert_eq!(vec![expected_member], members);
    }

    #[test]
    fn should_not_load_members_when_malformed_input() {
        let entry = get_malformed_member_as_csv();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(entry.as_bytes()));
        let members = load_memberships(&mut reader);
        assert!(members.is_empty(), "`members` is not empty.");
    }

    // endregion
    #[test]
    fn should_group_members_by_membership() {
        let jean = Membership::new(
            "1".to_string(),
            "Jean".to_string(),
            "".to_string(),
            None,
            None,
            "1".to_string(),
            "".to_string(),
            false,
            Default::default(),
            false,
            "".to_string(),
            "".to_string(),
        );

        let michel = Membership::new(
            "1".to_string(),
            "Michel".to_string(),
            "".to_string(),
            None,
            None,
            "1".to_string(),
            "".to_string(),
            false,
            Default::default(),
            false,
            "".to_string(),
            "".to_string(),
        );
        let pierre = Membership::new(
            "2".to_string(),
            "Pierre".to_string(),
            "".to_string(),
            None,
            None,
            "2".to_string(),
            "".to_string(),
            false,
            Default::default(),
            false,
            "".to_string(),
            "".to_string(),
        );

        let expected_map: Members = Members::from(
            [
                (
                    "1".to_owned(),
                    Memberships::from([jean.clone(), michel.clone()]),
                ),
                ("2".to_owned(), Memberships::from([pierre.clone()])),
            ]
            .into_iter()
            .collect::<HashMap<String, Memberships>>(),
        );
        let result = group_members_by_membership(vec![jean, pierre, michel]);
        assert_eq!(expected_map, result);
    }

    // region find_file
    #[test]
    fn should_find_file() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("memberships-{year}-{month:02}-{day:02}.csv"));
        File::create(&members_file).unwrap();

        let file_details = find_file(&temp_dir.into_os_string()).unwrap();
        assert_eq!(&members_file.into_os_string(), file_details.filepath());
        assert_eq!(
            &NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            file_details.update_date()
        );
    }

    #[test]
    fn should_not_find_file_when_no_file() {
        let temp_dir = temp_dir();
        let error = find_file(&temp_dir.into_os_string()).err().unwrap();
        assert!(matches!(error, ApplicationError::Membership(NoFileFound)));
    }

    #[test]
    fn should_not_find_file_when_no_file_match() {
        let year = 2025;
        let month = 2;
        let day = 1;
        let temp_dir = temp_dir();
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.pdf"));
        File::create(&members_file).unwrap();

        let error = find_file(&temp_dir.into_os_string()).err().unwrap();
        assert!(matches!(error, ApplicationError::Membership(NoFileFound)));
    }
    // endregion

    #[test]
    fn folder_should_exist() {
        let temp_dir = temp_dir();
        let result = check_folder(&temp_dir.into_os_string());
        assert!(result.is_ok());
    }

    #[test]
    fn folder_should_not_exist() {
        let error = check_folder(&OsString::from("/path/to/non/existing/folder")).unwrap_err();
        assert!(matches!(error, ApplicationError::Membership(NoFileFound)));
    }

    // region Conversions
    #[test]
    fn should_convert_captures_to_date() {
        let year = 2025;
        let month = 2;
        let day = 1;

        let string = OsString::from(format!("{year}-{month:02}-{day:02}"));
        let regex = Regex::new("(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})").unwrap();
        let captures = regex.captures(string.as_encoded_bytes()).unwrap();

        let result = convert_captures_to_date(&captures).unwrap();
        assert_eq!(NaiveDate::from_ymd_opt(year, month, day).unwrap(), result);
    }

    #[test]
    fn should_fail_to_convert_captures_to_date_when_invalid() {
        let year = 2025;
        let month = 22;
        let day = 1;

        let string = OsString::from(format!("{year}-{month:02}-{day:02}"));
        let regex = Regex::new("(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})").unwrap();
        let captures = regex.captures(string.as_encoded_bytes()).unwrap();

        let error = convert_captures_to_date(&captures).err().unwrap();
        assert!(matches!(error, ApplicationError::Membership(InvalidDate)));
    }

    #[test]
    fn should_convert_match_to_i32() {
        let int = 12;
        let message = OsString::from(int.to_string());
        let regex = Regex::new("(?<integer>\\d{2})").unwrap();

        let captures = regex.captures(message.as_encoded_bytes()).unwrap();
        let result: Result<i32, _> = convert_match_to_integer(&captures, "integer");
        assert_eq!(12, result.unwrap());
    }

    #[test]
    fn should_convert_match_to_u32() {
        let int = 12;
        let message = OsString::from(int.to_string());
        let regex = Regex::new("(?<integer>\\d{2})").unwrap();

        let captures = regex.captures(message.as_encoded_bytes()).unwrap();
        let result: Result<u32, _> = convert_match_to_integer(&captures, "integer");
        assert_eq!(12, result.unwrap());
    }

    #[test]
    fn should_fail_to_convert_match_when_wrong_format() {
        let message = OsString::from("ab");
        let regex = Regex::new("(?<integer>\\w{2})").unwrap();

        let captures = regex.captures(message.as_encoded_bytes()).unwrap();
        let error = convert_match_to_integer::<i32>(&captures, "integer").unwrap_err();
        assert!(matches!(
            error,
            ApplicationError::Membership(CantConvertDateFieldToString)
        ));
    }
    // endregion

    // region build_members_file_regex
    #[test]
    fn should_build_correct_members_file_regex() {
        let correct_file_name = OsString::from("memberships-2025-01-02.csv");
        let incorrect_file_name = OsString::from("2025-01-02.csv");
        let regex = build_members_file_regex().unwrap();

        let captures = regex
            .captures(correct_file_name.as_encoded_bytes())
            .unwrap();
        assert_eq!(
            4,
            captures.len(),
            "Regex should have captured 4 elements: the whole name and the 3 parts of the date."
        );
        let captures = regex.captures(incorrect_file_name.as_encoded_bytes());
        assert!(
            captures.is_none(),
            "Regex should not have captured anything."
        );
    }
    // endregion
}
