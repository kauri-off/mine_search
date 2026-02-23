use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::ips)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IpModel {
    pub id: i32,
    pub ip: String,
    pub port: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ips)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IpInsert<'a> {
    pub ip: &'a str,
    pub port: i32,
}
