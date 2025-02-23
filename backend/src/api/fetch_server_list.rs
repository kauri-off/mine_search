use super::fetch_server_info::ServerResponse;
use crate::database::DatabaseWrapper;
use axum::{extract::State, http::StatusCode, Json};
use db_schema::{models::servers::ServerModel, schema::*};
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

    // Если offset_ip указан, выбираем id для данного ip.
    // Иначе получаем максимальное значение id с помощью агрегатной функции max().
    // Обратите внимание, что тип id – i32, поэтому и возвращаемые типы должны быть i32.
    let server_id: i32 = if let Some(ref offset_ip) = body.offset_ip {
        servers::dsl::servers
            .filter(servers::dsl::ip.eq(offset_ip))
            .select(servers::dsl::id)
            .first::<i32>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        servers::dsl::servers
            .select(max(servers::dsl::id))
            .first::<Option<i32>>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or_default()
    };

    let filter: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = diesel::sql_types::Bool>> =
        match body.license {
            Some(t) => Box::new(servers::id.lt(server_id).and(servers::license.eq(t))),
            None => Box::new(servers::id.lt(server_id)),
        };

    let server_list = servers::table
        .left_join(players::table.on(players::server_id.eq(servers::id.nullable())))
        .filter(filter)
        .limit(body.limit)
        .group_by(servers::id)
        .select((ServerModel::as_select(), count(players::id).nullable()))
        .load::<(ServerModel, Option<i64>)>(&mut db.pool.get().await.unwrap())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        server_list
            .into_iter()
            .map(|(server, count)| ServerResponse::new(server, count.unwrap_or_default()))
            .collect(),
    ))
}
