use std::collections::{BTreeSet, HashMap};
use std::ffi::{OsStr};
use std::fs::File;
use std::str::FromStr;
use chrono::NaiveDate;
use regex::bytes::{Captures, Regex};
use crate::member::error::Error::{CantBrowseThroughFiles, CantConvertDateFieldToString, CantOpenMembersFile, NoFileFound, WrongRegex};
use crate::member::file_details::FileDetails;
use crate::member::Member;
use crate::member::Result;

pub fn import_from_file(filename: &OsStr) -> Result<HashMap<String, BTreeSet<Member>>> {
    let file = File::open(filename).map_err(|e| {
        error!("Can't open members file `{:?}`.\n{e:#?}", filename.to_str());
        CantOpenMembersFile
    })?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file);
    let members = reader.deserialize()
        .map(|result: Result<Member, _>| result.unwrap())
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
    let regex = Regex::new("^members-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$")
        .or(Err(WrongRegex))?;
    let paths = std::fs::read_dir("./").or(Err(CantBrowseThroughFiles))?;
    for path in paths {
        let path = path.expect("Path should be valid.");
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if captures.is_some() {
            let captures = captures.unwrap();
            let date = NaiveDate::from_ymd_opt(
                convert_match_to_integer(&captures, "year")?,
                convert_match_to_integer(&captures, "month")?,
                convert_match_to_integer(&captures, "day")?
            ).unwrap();

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