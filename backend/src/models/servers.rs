use chrono::Utc;
use diesel::prelude::*;
use serde_json::Value;

/// Manual "how do I get in" classification for a server, set by an operator after
/// probing it. Independent of `is_checked`/`is_crashed` and of the auto-detected
/// `requires_mods`. Postgres enum `join_status`; variants lowercase to the DB
/// labels via `diesel_derive_enum` (see [`crate::models::players::PlayerStatus`]).
#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::JoinStatus"]
pub enum JoinStatus {
    Undetermined,
    Spoofable,
    Whitelist,
    Password,
    Modded,
    Broken,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModel {
    pub id: i32,
    pub ip: String,
    pub port: i32,
    pub version_name: String,
    pub protocol: i32,
    pub description: Value,
    pub is_online_mode: bool,
    pub disconnect_reason: Option<Value>,
    pub is_checked: bool,
    pub join_status: JoinStatus,
    pub is_crashed: bool,
    pub is_online: bool,
    pub requires_mods: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub favicon: Option<String>,
    pub ping: Option<i64>,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModelMini {
    pub id: i32,
    pub ip: String,
    pub port: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerInsert<'a> {
    pub ip: &'a str,
    pub port: i32,
    pub version_name: &'a str,
    pub protocol: i32,
    pub description: &'a Value,
    /// Plaintext MOTD flattened from `description`, kept as a queryable column so
    /// free-text search can hit a trigram index instead of scanning JSONB.
    pub motd: &'a str,
    pub is_online_mode: bool,
    pub disconnect_reason: Option<Value>,
    pub requires_mods: bool,
    pub favicon: Option<&'a str>,
    pub ping: Option<i64>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerUpdate<'a> {
    pub version_name: &'a str,
    pub protocol: i32,
    pub description: &'a serde_json::Value,
    /// Plaintext MOTD flattened from `description` (see [`ServerInsert::motd`]).
    pub motd: &'a str,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_online: bool,
    pub requires_mods: bool,
    pub favicon: Option<&'a str>,
    pub ping: Option<i64>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerExtraUpdate {
    pub is_online_mode: bool,
    pub disconnect_reason: Option<Value>,
}
