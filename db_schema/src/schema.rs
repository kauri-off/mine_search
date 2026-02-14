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
        white_list -> Nullable<Bool>,
        checked -> Nullable<Bool>,
        auth_me -> Nullable<Bool>,
        crashed -> Nullable<Bool>,
        was_online -> Bool,
        created -> Timestamptz,
        updated -> Timestamptz,
    }
}

diesel::joinable!(data -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(data, servers,);
