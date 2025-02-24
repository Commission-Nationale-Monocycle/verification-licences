use std::collections::{BTreeSet, HashMap};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use chrono::NaiveDate;
use regex::bytes::Regex;
use crate::member::member::Member;

pub fn import_from_file(filename: &OsStr) -> HashMap<String, BTreeSet<Member>> {
    let file = match File::open(filename) {
        Ok(file) => { file }
        Err(e) => {
            error!("{e}");
            panic!("Can't find members file.")
        }
    };
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

    map
}

pub fn find_file() -> Option<(NaiveDate, OsString)> {
    let regex = Regex::new("^members-(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})\\.csv$").unwrap();
    let paths = std::fs::read_dir("./").unwrap();
    for path in paths {
        let path = path.unwrap();
        let filename = path.file_name();
        let captures = regex.captures(filename.as_encoded_bytes());
        if captures.is_some() {
            let captures = captures.unwrap();
            let date = NaiveDate::from_ymd_opt(
                String::from_utf8_lossy(&captures["year"]).parse::<i32>().unwrap(),
                String::from_utf8_lossy(&captures["month"]).parse::<u32>().unwrap(),
                String::from_utf8_lossy(&captures["day"]).parse::<u32>().unwrap(),
            ).unwrap();

            return Some((date, filename));
        }
    }
    None
}