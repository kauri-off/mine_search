use super::fetch_server_info::ServerResponse;
use crate::database::DatabaseWrapper;
use crate::database::ServerModelWithPlayers;
use axum::{extract::State, http::StatusCode, Json};
use db_schema::schema::*;
use diesel::dsl::*;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub limit: i64,
    pub offset_ip: Option<String>,
    pub license: Option<bool>,
    pub has_players: Option<bool>,
    pub white_list: Option<bool>,
}

pub async fn fetch_server_list(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<Vec<ServerResponse>>, StatusCode> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let server_id: i32 = if let Some(ref offset_ip) = body.offset_ip {
        servers::table
            .filter(servers::ip.eq(offset_ip))
            .select(servers::id)
            .first::<i32>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        servers::table
            .select(max(servers::id))
            .first::<Option<i32>>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or_default()
    };

    let license_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.license {
        Some(license) => Box::new(servers::license.eq(license)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let white_list_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.white_list
    {
        Some(white_list) => Box::new(servers::white_list.assume_not_null().eq(white_list)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let players_count_filter = match body.has_players {
        Some(true) => "COUNT(players.id) > 0",
        Some(false) => "COUNT(players.id) = 0",
        None => "1 = 1", // Всегда истинное условие, фактически убирает фильтр
    };

    let server_list = servers::table
        .filter(servers::id.lt(server_id))
        .filter(license_filter)
        .filter(white_list_filter)
        .order(servers::id.desc())
        .left_join(players::table.on(players::server_id.eq(servers::id.nullable())))
        .limit(body.limit)
        .group_by(servers::id)
        .having(diesel::dsl::sql::<Bool>(players_count_filter))
        .select((
            servers::id,
            servers::ip,
            servers::online,
            servers::max,
            servers::version_name,
            servers::protocol,
            servers::license,
            servers::white_list,
            servers::last_seen,
            servers::description,
            servers::was_online,
            count(players::id).nullable(),
        ))
        .load::<ServerModelWithPlayers>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(server_list.into_iter().map(Into::into).collect()))
}
