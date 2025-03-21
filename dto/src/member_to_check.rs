use crate::member_identifier::MemberIdentifier;

pub trait MemberToCheck: MemberIdentifier {
    fn id(&self) -> Option<u16>;
    fn first_name(&self) -> String;
    fn last_name(&self) -> String;
    fn email(&self) -> Option<String>;
    fn club(&self) -> Option<String>;
    fn confirmed(&self) -> Option<bool>;
}
