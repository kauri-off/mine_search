// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "player_status"))]
    pub struct PlayerStatus;
}

diesel::table! {
    ping_requests (id) {
        id -> Int4,
        server_id -> Int4,
        with_connection -> Bool,
    }
}

diesel::table! {
    player_count_snapshots (id) {
        id -> Int8,
        server_id -> Int4,
        players_online -> Int4,
        players_max -> Int4,
        recorded_at -> Timestamptz,
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
    scan_targets (id) {
        id -> Int4,
        ip -> Text,
        port -> Int4,
        quick -> Bool,
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
        is_online_mode -> Bool,
        is_checked -> Bool,
        is_spoofable -> Nullable<Bool>,
        is_crashed -> Bool,
        is_online -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        disconnect_reason -> Nullable<Jsonb>,
        is_forge -> Bool,
        favicon -> Nullable<Text>,
    }
}

diesel::joinable!(ping_requests -> servers (server_id));
diesel::joinable!(player_count_snapshots -> servers (server_id));
diesel::joinable!(players -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(
    ping_requests,
    player_count_snapshots,
    players,
    scan_targets,
    servers,
);
