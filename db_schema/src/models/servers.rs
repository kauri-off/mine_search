use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModel {
    pub id: i32,
    pub ip: String,
    pub online: i32,
    pub max: i32,
    pub version_name: String,
    pub protocol: i32,
    pub license: bool,
    pub white_list: Option<bool>,
    pub last_seen: NaiveDateTime,
    pub description: Value,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModelMini {
    pub id: i32,
    pub ip: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerInsert<'a> {
    pub ip: &'a str,
    pub online: i32,
    pub max: i32,
    pub version_name: &'a str,
    pub protocol: i32,
    pub license: bool,
    pub white_list: Option<bool>,
    pub description: &'a Value,
    pub was_online: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerUpdate<'a> {
    pub online: i32,
    pub max: i32,
    pub version_name: &'a str,
    pub protocol: i32,
    pub description: &'a Value,
    pub was_online: bool,
    pub last_seen: NaiveDateTime,
}
