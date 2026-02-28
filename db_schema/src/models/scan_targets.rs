use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::scan_targets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TargetModel {
    pub id: i32,
    pub ip: String,
    pub port: i32,
    pub quick: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::scan_targets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TargetInsert<'a> {
    pub ip: &'a str,
    pub port: i32,
    pub quick: bool,
}
