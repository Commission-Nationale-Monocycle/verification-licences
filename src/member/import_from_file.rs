use std::collections::{BTreeSet, HashMap};
use std::ffi::{OsStr};
use std::fs::File;
use std::str::FromStr;
use chrono::NaiveDate;
use regex::bytes::{Captures, Regex};
use crate::member::error::Error::{CantBrowseThroughFiles, CantConvertDateFieldToString, CantOpenMembersFile, CantOpenMembersFileFolder, InvalidDate, NoFileFound, WrongRegex};
use crate::member::file_details::FileDetails;
use crate::member::{Member, MEMBERS_FILE_FOLDER};
use crate::member::Result;
use crate::tools::log_message;

pub fn import_from_file(filename: &OsStr) -> Result<HashMap<String, BTreeSet<Member>>> {
    let file = File::open(filename).map_err(|e| {
        error!("Can't open members file `{:?}`.\n{e:#?}", filename.to_str());
        CantOpenMembersFile
    })?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file);
    let members = reader.deserialize()
        .filter_map(|result: Result<Member, _>| match result {
            Ok(member) => Some(member),
            Err(e) => {
                log_message("Error while reading member")(e);
                None
            }
        })
        .collect::<Vec<_>>();

    let mut map = HashMap::new();

    for member in members {
        let membership_number = member.membership_number().to_string();
        map.entry(membership_number)
            .and_modify(|set: &mut BTreeSet<Member>| { set.insert(member.clone()); })
            .or_insert(BTreeSet::from([member.clone(); 1]));
    }

    Ok(map)
}

pub fn find_file() -> Result<FileDetails> {
    match std::fs::exists(MEMBERS_FILE_FOLDER) {
        Ok(true) => Ok(()),
        Ok(false) => Err(NoFileFound),
        Err(e) => {
            error!("MEMBERS_FILE_FOLDER `{MEMBERS_FILE_FOLDER} is inaccessible.\n{e:#?}");
            Err(CantOpenMembersFileFolder)
        }
    }?;

    let regex = Regex::new("^members-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$")
        .or(Err(WrongRegex))?;
    let paths = std::fs::read_dir(MEMBERS_FILE_FOLDER).or(Err(CantBrowseThroughFiles))?;
    for path in paths {
        let path = path.expect("Path should be valid.");
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if let Some(captures) = captures {
            let date = NaiveDate::from_ymd_opt(
                convert_match_to_integer(&captures, "year")?,
                convert_match_to_integer(&captures, "month")?,
                convert_match_to_integer(&captures, "day")?
            ).ok_or(InvalidDate)?;

            return Ok(FileDetails::new(date, filename));
        }
    }

    Err(NoFileFound)
}

fn convert_match_to_integer<T: FromStr>(captures: &Captures, key: &str) -> Result<T> {
    String::from_utf8_lossy(&captures[key])
        .parse::<T>()
        .or(Err(CantConvertDateFieldToString))
}