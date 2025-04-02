// @generated automatically by Diesel CLI.

diesel::table! {
    membership (id) {
        id -> Integer,
        last_name -> Text,
        first_name -> Text,
        gender -> Text,
        birthdate -> Nullable<Text>,
        age -> Nullable<Integer>,
        membership_number -> Text,
        email_address -> Text,
        payed -> Bool,
        end_date -> Text,
        expired -> Bool,
        club -> Text,
        structure_code -> Text,
    }
}
