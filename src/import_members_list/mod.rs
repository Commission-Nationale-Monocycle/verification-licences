mod member;

use std::collections::HashMap;
use std::fs::File;
use crate::import_members_list::member::Member;

pub fn import_from_file(filename: &str) -> HashMap<String, Member> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(File::open(filename).unwrap());
    reader.deserialize()
        .map(|result: Result<Member, _>| result.unwrap())
        .map(|member| (member.membership_number.to_string(), member))
        .collect()
}