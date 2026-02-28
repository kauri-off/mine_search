use chrono::Utc;
use diesel::prelude::*;
use serde_json::Value;

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
    pub is_spoofable: Option<bool>,
    pub is_crashed: bool,
    pub is_online: bool,
    pub is_forge: bool,
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
    pub is_online_mode: bool,
    pub disconnect_reason: Option<Value>,
    pub is_forge: bool,
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
    pub updated_at: chrono::DateTime<Utc>,
    pub is_online: bool,
    pub is_forge: bool,
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
