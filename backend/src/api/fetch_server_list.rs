use super::fetch_server_info::ServerResponse;
use crate::database::DatabaseWrapper;
use axum::{extract::State, http::StatusCode, Json};
use db_schema::{models::servers::ServerModel, schema::servers::dsl::*};
use diesel::{dsl::max, ExpressionMethods, QueryDsl, SelectableHelper};
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
        servers
            .filter(ip.eq(offset_ip))
            .select(id)
            .first::<i32>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        servers
            .select(max(id))
            .first::<Option<i32>>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or_default()
    };

    let mut query = servers.into_boxed().filter(id.lt(server_id));

    if let Some(license_filter) = body.license {
        query = query.filter(license.eq(license_filter));
    }

    let server_list: Vec<ServerModel> = query
        .limit(body.limit)
        .order(id.desc())
        .select(ServerModel::as_select())
        .load(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(server_list.into_iter().map(Into::into).collect()))
}
