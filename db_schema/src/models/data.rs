use chrono::Utc;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::data)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataModel {
    pub id: i64,
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::data)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DataInsert {
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
}
