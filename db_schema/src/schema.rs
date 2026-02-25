// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "player_status"))]
    pub struct PlayerStatus;
}

diesel::table! {
    data (id) {
        id -> Int8,
        server_id -> Int4,
        online -> Int4,
        max -> Int4,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    ips (id) {
        id -> Int4,
        ip -> Text,
        port -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PlayerStatus;

    players (id) {
        id -> Int4,
        server_id -> Int4,
        name -> Varchar,
        status -> PlayerStatus,
    }
}

diesel::table! {
    server_ping (id) {
        id -> Int4,
        server_id -> Int4,
        with_connection -> Bool,
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
        disconnect_reason -> Nullable<Jsonb>,
        is_modded -> Bool,
    }
}

diesel::joinable!(data -> servers (server_id));
diesel::joinable!(players -> servers (server_id));
diesel::joinable!(server_ping -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(data, ips, players, server_ping, servers,);
