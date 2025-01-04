use std::env;

use chrono::NaiveDateTime;
use db_schema::schema;
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct DatabaseWrapper {
    pub pool: Pool<AsyncPgConnection>,
}

impl DatabaseWrapper {
    pub fn establish() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().unwrap();

        Self { pool }
    }
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = schema::servers)]
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
    pub description: Value,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::players)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerModel {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub last_seen: NaiveDateTime,
    pub server_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::servers)]
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
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = schema::servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerUpdate<'a> {
    pub online: i32,
    pub max: i32,
    pub version_name: &'a str,
    pub protocol: i32,
    pub description: &'a Value,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerInsert<'a> {
    pub uuid: &'a str,
    pub name: &'a str,
    pub server_id: i32,
}
