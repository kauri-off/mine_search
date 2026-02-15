use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::servers::ServerModel;

#[derive(Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::data)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataModel {
    pub id: i64,
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
    pub players: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::data)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataInsert<'a> {
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
    pub players: &'a serde_json::Value,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataModelMini {
    pub id: i64,
    pub players: serde_json::Value,
}
