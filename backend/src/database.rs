use std::env;

use chrono::NaiveDateTime;
use diesel::prelude::Queryable;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
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

#[allow(dead_code)]
#[derive(Queryable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerModelWithPlayers {
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
    pub was_online: bool,
    pub player_count: Option<i64>,
}
