use derive_getters::Getters;
use dto::uda::Participant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Getters, Serialize, Deserialize, PartialEq)]
pub struct ImportedParticipant {
    #[serde(rename = "Id")]
    id: u16,
    #[serde(rename = "Manual Organization Membership#")]
    manual_organization_membership: Option<String>,
    #[serde(rename = "System Organization Membership#")]
    system_organization_membership: Option<String>,
    #[serde(rename = "First Name")]
    first_name: String,
    #[serde(rename = "Last Name")]
    last_name: String,
    #[serde(rename = "Birthday")]
    birthday: String,
    #[serde(rename = "Address Line1")]
    address_line: String,
    #[serde(rename = "City")]
    city: String,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Zip")]
    zip: String,
    #[serde(rename = "Country")]
    country: String,
    #[serde(rename = "Phone")]
    phone: Option<String>,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Club")]
    club: Option<String>,
    #[serde(rename = "Confirmed already a member")]
    confirmed: bool,
}

impl From<ImportedParticipant> for Participant {
    fn from(imported_participant: ImportedParticipant) -> Self {
        Participant::new(
            imported_participant.id,
            imported_participant
                .manual_organization_membership
                .or(imported_participant.system_organization_membership),
            imported_participant.first_name,
            imported_participant.last_name,
            imported_participant.email,
            imported_participant.club,
            imported_participant.confirmed,
        )
    }
}

#[cfg(test)]
impl ImportedParticipant {
    pub fn new(
        id: u16,
        manual_organization_membership: Option<String>,
        system_organization_membership: Option<String>,
        first_name: String,
        last_name: String,
        birthday: String,
        address_line: String,
        city: String,
        state: Option<String>,
        zip: String,
        country: String,
        phone: Option<String>,
        email: String,
        club: Option<String>,
        confirmed: bool,
    ) -> Self {
        Self {
            id,
            manual_organization_membership,
            system_organization_membership,
            first_name,
            last_name,
            birthday,
            address_line,
            city,
            state,
            zip,
            country,
            phone,
            email,
            club,
            confirmed,
        }
    }
}
