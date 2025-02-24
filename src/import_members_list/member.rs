use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Member {
    #[serde(alias = "Nom d'usage")]
    name: String,
    #[serde(alias = "Prénom")]
    firstname: String,
    #[serde(alias = "Sexe")]
    gender: String,
    #[serde(alias = "Date de Naissance")]
    birthdate: String,
    #[serde(alias = "Age")]
    age: String,
    #[serde(alias = "Email")]
    email_address: String,
    #[serde(alias = "Réglé")]
    payed: String,
    #[serde(alias = "Date Fin d'adhésion")]
    end_date: String,
    #[serde(alias = "Adherent expiré")]
    expired: String,
    #[serde(alias = "Nom de structure")]
    club: String,
    #[serde(alias = "Code de structure")]
    membership_number: String
}