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
    pub license: bool,
    pub disconnect_reason: Option<Value>,
    pub checked: bool,
    pub spoofable: Option<bool>,
    pub crashed: bool,
    pub was_online: bool,
    pub is_forge: bool,
    pub created: chrono::DateTime<Utc>,
    pub updated: chrono::DateTime<Utc>,
    pub favicon: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable)]
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
    pub license: bool,
    pub disconnect_reason: Option<Value>,
    pub is_forge: bool,
    pub favicon: Option<&'a str>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerUpdate<'a> {
    pub version_name: &'a str,
    pub protocol: i32,
    pub description: &'a serde_json::Value,
    pub updated: chrono::DateTime<Utc>,
    pub was_online: bool,
    pub is_forge: bool,
    pub favicon: Option<&'a str>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerExtraUpdate {
    pub license: bool,
    pub disconnect_reason: Option<Value>,
}
