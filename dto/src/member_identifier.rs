pub trait MemberIdentifier: PartialOrd + PartialEq + Clone {
    fn membership_num(&self) -> Option<String>;
}
