// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "player_status"))]
    pub struct PlayerStatus;
}

diesel::table! {
    player_count_snapshots (server_id, recorded_at) {
        server_id -> Int4,
        players_online -> Int2,
        players_max -> Int2,
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
        last_seen_at -> Timestamptz,
    }
}

diesel::table! {
    processed_results (result_id) {
        result_id -> Text,
        processed_at -> Timestamptz,
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
        requires_mods -> Bool,
        favicon -> Nullable<Text>,
        ping -> Nullable<Int8>,
    }
}

diesel::joinable!(player_count_snapshots -> servers (server_id));
diesel::joinable!(players -> servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(
    player_count_snapshots,
    players,
    processed_results,
    servers,
);
