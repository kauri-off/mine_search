use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::server_ping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerPingModel {
    pub id: i32,
    pub server_id: i32,
    pub with_connection: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::server_ping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServerPingInsert {
    pub server_id: i32,
    pub with_connection: bool,
}
