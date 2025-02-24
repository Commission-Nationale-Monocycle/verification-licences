use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use chrono::NaiveDate;
use derive_getters::Getters;
use crate::member::member::Member;

#[derive(Getters, Default)]
pub struct MembersState {
    filename: Option<OsString>,
    last_update: Option<NaiveDate>,
    members: HashMap<String, BTreeSet<Member>>
}

impl MembersState {
    pub fn new(members_filename: Option<OsString>,
               last_update: Option<NaiveDate>,
               members: HashMap<String, BTreeSet<Member>>) -> Self {
        Self { filename: members_filename, last_update, members }
    }

    pub fn set_filename(&mut self, filename: OsString) {
        self.filename = Some(filename);
    }

    pub fn set_last_update(&mut self, last_update: NaiveDate) {
        self.last_update = Some(last_update);
    }
}