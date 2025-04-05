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
        normalized_membership_number -> Text,
        normalized_last_name -> Text,
        normalized_first_name -> Text,
        normalized_last_name_first_name -> Text,
        normalized_first_name_last_name -> Text,
    }
}

diesel::table! {
    uda_instance (id) {
        id -> Integer,
        slug -> Text,
        name -> Text,
        url -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(last_update, membership, uda_instance,);
