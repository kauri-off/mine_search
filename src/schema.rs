// @generated automatically by Diesel CLI.

diesel::table! {
    players (id) {
        id -> Integer,
        uuid -> Text,
        name -> Text,
        last_seen -> Timestamp,
        server_id -> Integer,
    }
}

diesel::table! {
    servers (id) {
        id -> Integer,
        ip -> Text,
        online -> Integer,
        max -> Integer,
        version_name -> Text,
        protocol -> Integer,
        license -> Bool,
        white_list -> Nullable<Bool>,
        description -> Nullable<Text>,
    }
}

diesel::joinable!(players -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(players, servers,);
