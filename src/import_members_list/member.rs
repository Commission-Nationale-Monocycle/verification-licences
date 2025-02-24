use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Member {
    #[serde(alias = "Nom d'usage")]
    pub name: String,
    #[serde(alias = "Prénom")]
    pub firstname: String,
    #[serde(alias = "Sexe")]
    pub gender: String,
    #[serde(alias = "Date de Naissance")]
    pub birthdate: String,
    #[serde(alias = "Age")]
    pub age: String,
    #[serde(alias = "Email")]
    pub email_address: String,
    #[serde(alias = "Réglé")]
    pub payed: String,
    #[serde(alias = "Date Fin d'adhésion")]
    pub end_date: String,
    #[serde(alias = "Adherent expiré")]
    pub expired: String,
    #[serde(alias = "Nom de structure")]
    pub club: String,
    #[serde(alias = "Code de structure")]
    pub membership_number: String
}