use chrono::Utc;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::player_count_snapshots)]
#[diesel(primary_key(server_id, recorded_at))]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SnapshotModel {
    pub server_id: i32,
    pub players_online: i16,
    pub players_max: i16,
    pub recorded_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::player_count_snapshots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SnapshotInsert {
    pub server_id: i32,
    pub players_online: i16,
    pub players_max: i16,
}
