use std::collections::{BTreeSet, HashMap};
use derive_getters::Getters;
use crate::member::file_details::FileDetails;
use crate::member::Member;

#[derive(Getters, Default)]
pub struct MembersState {
    file_details: Option<FileDetails>,
    members: HashMap<String, BTreeSet<Member>>,
}

impl MembersState {
    pub fn new(file_details: Option<FileDetails>) -> Self {
        Self { file_details, members: HashMap::new() }
    }

    pub fn set_file_details(&mut self, file_details: FileDetails) {
        self.file_details = Some(file_details);
    }

    pub fn set_members(&mut self, members: HashMap<String, BTreeSet<Member>>) {
        self.members = members;
    }
}
