// @generated automatically by Diesel CLI.

diesel::table! {
    last_update (element) {
        element -> Text,
        date -> Text,
    }
}

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

diesel::allow_tables_to_appear_in_same_query!(last_update, membership,);
