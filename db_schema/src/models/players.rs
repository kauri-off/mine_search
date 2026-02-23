use diesel::prelude::*;

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::PlayerStatus"]
pub enum PlayerStatus {
    None,
    Regular,
    Admin,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerModel {
    pub id: i32,
    pub server_id: i32,
    pub name: String,
    pub status: PlayerStatus,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerModelMini {
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::players)]
#[diesel(belongs_to(ServerModel, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerInsert<'a> {
    pub server_id: i32,
    pub name: &'a str,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerUpdate<'a> {
    pub status: &'a PlayerStatus,
}
