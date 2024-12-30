use std::env;

use chrono::NaiveDateTime;
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Connection, Selectable, SqliteConnection,
};
use dotenvy::dotenv;

pub struct DatabaseWrapper {
    pub conn: SqliteConnection,
}

impl DatabaseWrapper {
    pub fn establish() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self { conn }
    }
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::server)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ServerModel {
    pub id: i32,
    pub ip: String,
    pub online: i32,
    pub max: i32,
    pub version_name: String,
    pub protocol: i32,
    pub license: bool,
    pub white_list: Option<bool>,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = crate::schema::players)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PlayersModel {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub last_seen: NaiveDateTime,
    pub server_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::server)]
pub struct ServerInsert<'a> {
    pub ip: &'a str,
    pub online: i32,
    pub max: i32,
    pub version_name: &'a str,
    pub protocol: i32,
    pub license: bool,
    pub white_list: Option<bool>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::players)]
pub struct PlayerInsert<'a> {
    pub uuid: &'a str,
    pub name: &'a str,
    pub server_id: i32,
}
