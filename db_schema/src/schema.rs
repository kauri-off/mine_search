// @generated automatically by Diesel CLI.

diesel::table! {
    data (id) {
        id -> Int8,
        server_id -> Int4,
        online -> Int4,
        max -> Int4,
        players -> Jsonb,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    servers (id) {
        id -> Int4,
        ip -> Varchar,
        port -> Int4,
        version_name -> Varchar,
        protocol -> Int4,
        description -> Jsonb,
        license -> Bool,
        checked -> Bool,
        spoofable -> Nullable<Bool>,
        crashed -> Bool,
        was_online -> Bool,
        created -> Timestamptz,
        updated -> Timestamptz,
        unique_players -> Int4,
        disconnect_reason -> Nullable<Jsonb>,
    }
}

diesel::joinable!(data -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(data, servers,);
