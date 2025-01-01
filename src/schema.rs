// @generated automatically by Diesel CLI.

diesel::table! {
    players (id) {
        id -> Int4,
        uuid -> Text,
        name -> Text,
        last_seen -> Timestamp,
        server_id -> Nullable<Int4>,
    }
}

diesel::table! {
    servers (id) {
        id -> Int4,
        ip -> Text,
        online -> Int4,
        max -> Int4,
        version_name -> Text,
        protocol -> Int4,
        license -> Bool,
        white_list -> Nullable<Bool>,
        description -> Nullable<Text>,
    }
}

diesel::joinable!(players -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(
    players,
    servers,
);
