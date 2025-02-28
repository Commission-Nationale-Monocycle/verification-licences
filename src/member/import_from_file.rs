use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::str::FromStr;

use chrono::NaiveDate;
use csv::Reader;
use regex::bytes::{Captures, Regex};

use crate::member::{Membership, MembershipDto};
use crate::member::error::Error::{CantBrowseThroughFiles, CantConvertDateFieldToString, CantOpenMembersFile, CantOpenMembersFileFolder, InvalidDate, NoFileFound, WrongRegex};
use crate::member::file_details::FileDetails;
use crate::member::memberships::Memberships;
use crate::member::members::Members;
use crate::member::Result;
use crate::tools::{log_message, log_message_and_return};

pub fn import_from_file(filepath: &OsStr) -> Result<Members> {
    let error_message = format!("Can't open members file `{:?}`.", filepath.to_str());
    let error_mapping = log_message_and_return(
        &error_message,
        CantOpenMembersFile,
    );
    let file = File::open(filepath).map_err(error_mapping)?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file);
    let members = load_members(&mut reader);
    Ok(group_members_by_membership(members))
}

fn load_members<T>(reader: &mut Reader<T>) -> Vec<MembershipDto> where T: std::io::Read {
    reader.deserialize()
        .filter_map(|result: Result<Membership, _>| match result {
            Ok(member) => Some(member.into()),
            Err(e) => {
                log_message("Error while reading member")(e);
                None
            }
        })
        .collect::<Vec<_>>()
}

fn group_members_by_membership(members: Vec<MembershipDto>) -> Members {
    let mut map = HashMap::new();

    members.into_iter()
        .for_each(|member| {
            let membership_number = member.membership_number().to_string();
            map.entry(membership_number)
                .and_modify(|memberships: &mut Memberships| { memberships.insert(member.clone()); })
                .or_insert(Memberships::from([member.clone(); 1]));
        });

    Members::from(map)
}

pub fn find_file(members_file_folder: &OsStr) -> Result<FileDetails> {
    check_folder(members_file_folder)?;

    let regex = Regex::new("^members-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$")
        .or(Err(WrongRegex))?;
    let paths = std::fs::read_dir(members_file_folder).or(Err(CantBrowseThroughFiles))?;
    for path in paths {
        let path = path.expect("Path should be valid.");
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if let Some(captures) = captures {
            let date = convert_captures_to_date(&captures)?;
            return Ok(FileDetails::new(date, path.path().into_os_string()));
        }
    }

    Err(NoFileFound)
}

fn check_folder(members_file_folder: &OsStr) -> Result<()> {
    match std::fs::exists(members_file_folder) {
        Ok(true) => Ok(()),
        Ok(false) => Err(NoFileFound),
        Err(e) => {
            log_message(&format!("`{members_file_folder:?}` folder is inaccessible."))(e);
            Err(CantOpenMembersFileFolder)
        }
    }
}

fn convert_captures_to_date(captures: &Captures) -> Result<NaiveDate> {
    NaiveDate::from_ymd_opt(
        convert_match_to_integer(captures, "year")?,
        convert_match_to_integer(captures, "month")?,
        convert_match_to_integer(captures, "day")?,
    ).ok_or(InvalidDate)
}

fn convert_match_to_integer<T: FromStr>(captures: &Captures, key: &str) -> Result<T> {
    String::from_utf8_lossy(&captures[key])
        .parse::<T>()
        .or(Err(CantConvertDateFieldToString))
}

/// Clean all files in folder that aren't given file.
pub fn clean_old_files(members_file_folder: &OsStr, file_update_date: &NaiveDate) -> Result<()> {
    let regex = build_members_file_regex()?;
    let paths = fs::read_dir(members_file_folder).or(Err(CantBrowseThroughFiles))?;
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
    Regex::new("^members-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$").or(Err(WrongRegex))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;

    use chrono::NaiveDate;
    use regex::bytes::Regex;

    use crate::member::error::Error::{CantConvertDateFieldToString, CantOpenMembersFile, InvalidDate, NoFileFound};
    use crate::member::import_from_file::{build_members_file_regex, check_folder, convert_captures_to_date, convert_match_to_integer, find_file, group_members_by_membership, import_from_file, load_members};
    use crate::member::memberships::Memberships;
    use crate::member::MembershipDto;
    use crate::member::members::Members;
    use crate::member::tests::{get_expected_member, get_malformed_member_as_csv, get_member_as_csv};
    use crate::tools::test::tests::temp_dir;

    // region import_from_file
    #[test]
    fn should_import_from_file() {
        let dir = temp_dir();
        let file_name = "members.csv";
        let file_path = dir.join(file_name);

        fs::write(&file_path, get_member_as_csv()).unwrap();

        let result = import_from_file(file_path.as_ref()).unwrap();
        assert_eq!(&Memberships::from([get_expected_member()]), result.get("123456").unwrap())
    }

    #[test]
    fn should_not_import_from_file_when_cant_open_file() {
        let dir = temp_dir();
        let file_name = "members.csv";
        let file_path = dir.join(file_name);

        let result = import_from_file(file_path.as_ref()).err().unwrap();
        assert_eq!(CantOpenMembersFile, result)
    }

    // endregion

    // region load_members
    #[test]
    fn should_load_members() {
        let entry = get_member_as_csv();
        let expected_member = get_expected_member();

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(entry.as_bytes()));
        let members = load_members(&mut reader);
        assert_eq!(vec![expected_member], members);
    }

    #[test]
    fn should_not_load_members_when_malformed_input() {
        let entry = get_malformed_member_as_csv();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(entry.as_bytes()));
        let members = load_members(&mut reader);
        assert!(members.is_empty(), "`members` is not empty.");
    }

    // endregion
    #[test]
    fn should_group_members_by_membership() {
        let jean = MembershipDto {
            name: "1".to_string(),
            firstname: "Jean".to_string(),
            gender: "".to_string(),
            birthdate: None,
            age: None,
            membership_number: "1".to_string(),
            email_address: "".to_string(),
            payed: false,
            end_date: Default::default(),
            expired: false,
            club: "".to_string(),
            structure_code: "".to_string(),
        };

        let michel = MembershipDto {
            name: "1".to_string(),
            firstname: "Michel".to_string(),
            gender: "".to_string(),
            birthdate: None,
            age: None,
            membership_number: "1".to_string(),
            email_address: "".to_string(),
            payed: false,
            end_date: Default::default(),
            expired: false,
            club: "".to_string(),
            structure_code: "".to_string(),
        };
        let pierre = MembershipDto {
            name: "2".to_string(),
            firstname: "Pierre".to_string(),
            gender: "".to_string(),
            birthdate: None,
            age: None,
            membership_number: "2".to_string(),
            email_address: "".to_string(),
            payed: false,
            end_date: Default::default(),
            expired: false,
            club: "".to_string(),
            structure_code: "".to_string(),
        };

        let expected_map: Members = Members::from([
            ("1".to_owned(), Memberships::from([jean.clone(), michel.clone()])),
            ("2".to_owned(), Memberships::from([pierre.clone()])),
        ].into_iter().collect::<HashMap<String, Memberships>>());
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
        let members_file = temp_dir.join(format!("members-{year}-{month:02}-{day:02}.csv"));
        File::create(&members_file).unwrap();

        let file_details = find_file(&temp_dir.into_os_string()).unwrap();
        assert_eq!(&members_file.into_os_string(), file_details.filepath());
        assert_eq!(&NaiveDate::from_ymd_opt(year, month, day).unwrap(), file_details.update_date());
    }

    #[test]
    fn should_not_find_file_when_no_file() {
        let temp_dir = temp_dir();
        let error = find_file(&temp_dir.into_os_string()).err().unwrap();
        assert_eq!(NoFileFound, error);
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
        assert_eq!(NoFileFound, error);
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
        let result = check_folder(&OsString::from("/path/to/non/existing/folder"));
        assert_eq!(NoFileFound, result.err().unwrap());
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

        let result = convert_captures_to_date(&captures).err().unwrap();
        assert_eq!(InvalidDate, result);
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
        let result: Result<u32, _> = convert_match_to_integer(&captures, "integer");
        assert_eq!(Err(CantConvertDateFieldToString), result);
    }
    // endregion

    // region build_members_file_regex
    #[test]
    fn should_build_correct_members_file_regex() {
        let correct_file_name = OsString::from("members-2025-01-02.csv");
        let incorrect_file_name = OsString::from("2025-01-02.csv");
        let regex = build_members_file_regex().unwrap();

        let captures = regex.captures(correct_file_name.as_encoded_bytes()).unwrap();
        assert_eq!(4, captures.len(),
                   "Regex should have captured 4 elements: the whole name and the 3 parts of the date.");
        let captures = regex.captures(incorrect_file_name.as_encoded_bytes());
        assert!(captures.is_none(), "Regex should not have captured anything.");
    }
    // endregion
}