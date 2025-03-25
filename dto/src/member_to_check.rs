pub trait MemberToCheck: PartialOrd + PartialEq + Clone {
    fn id(&self) -> Option<u16>;
    fn membership_num(&self) -> Option<String>;
    fn identity(&self) -> Option<String>;
    fn first_name(&self) -> Option<String>;
    fn last_name(&self) -> Option<String>;
    fn email(&self) -> Option<String>;
    fn club(&self) -> Option<String>;
    fn confirmed(&self) -> Option<bool>;
}
