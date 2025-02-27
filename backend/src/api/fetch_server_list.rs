use super::fetch_server_info::ServerResponse;
use crate::database::DatabaseWrapper;
use crate::database::ServerModelWithPlayers;
use axum::{extract::State, http::StatusCode, Json};
use db_schema::schema::*;
use diesel::dsl::*;
use diesel::prelude::*;
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

    let mut filter: Box<
        dyn BoxableExpression<
            _,
            diesel::pg::Pg,
            SqlType = diesel::sql_types::Nullable<diesel::sql_types::Bool>,
        >,
    > = Box::new(servers::id.lt(server_id).nullable());

    if let Some(license) = body.license {
        filter = Box::new(filter.and(servers::license.eq(license)));
    }

    if let Some(white_list) = body.white_list {
        filter = Box::new(filter.and(servers::white_list.eq(white_list)));
    }

    let players_count_filter_value = match body.has_players {
        Some(true) => 0,
        Some(false) => -1,
        None => -1,
    };

    let server_list = servers::table
        .filter(filter)
        .order(servers::id.desc())
        .left_join(players::table.on(players::server_id.eq(servers::id.nullable())))
        .limit(body.limit)
        .group_by(servers::id)
        .having(count(players::id).gt(players_count_filter_value))
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
