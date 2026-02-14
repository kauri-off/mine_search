use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModel {
    pub id: i32,
    pub ip: String,
    pub port: i32,
    pub version_name: String,
    pub protocol: i32,
    pub description: serde_json::Value,
    pub license: bool,
    pub white_list: Option<bool>,
    pub checked: Option<bool>,
    pub auth_me: Option<bool>,
    pub crashed: Option<bool>,
    pub was_online: bool,
    pub created: chrono::DateTime<Utc>,
    pub updated: chrono::DateTime<Utc>,
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
    pub description: &'a serde_json::Value,
    pub license: bool,
    pub white_list: Option<bool>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerUpdate<'a> {
    pub description: &'a serde_json::Value,
    pub updated: chrono::DateTime<Utc>,
    pub was_online: bool,
}
