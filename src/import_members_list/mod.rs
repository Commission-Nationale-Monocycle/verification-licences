mod member;

use std::fs::File;
use crate::import_members_list::member::Member;

pub fn import_from_file(filename: &str) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(File::open(filename).unwrap());
    for result in reader.deserialize() {
        let record: Member = result.unwrap();
        println!("{:?}", record);
    }
}