use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::servers::ServerModel;

#[derive(Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::players)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerModel {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub last_seen: NaiveDateTime,
    pub server_id: Option<i32>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerInsert<'a> {
    pub uuid: &'a str,
    pub name: &'a str,
    pub server_id: i32,
}
