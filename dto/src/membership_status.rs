use crate::membership::Membership;
use crate::membership_status::MemberStatus::{Expired, Unknown, UpToDate};
use chrono::Utc;

#[derive(Debug, Eq, PartialEq)]
pub enum MemberStatus {
    UpToDate,
    Expired,
    Unknown,
}

pub fn compute_member_status(membership: Option<&Membership>) -> MemberStatus {
    match membership {
        None => Unknown,
        Some(membership) => {
            if Utc::now().date_naive() <= *membership.end_date() {
                UpToDate
            } else {
                Expired
            }
        }
    }
}
