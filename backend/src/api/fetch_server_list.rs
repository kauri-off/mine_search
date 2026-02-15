use super::fetch_server_info::ServerResponse;
use crate::database::DatabaseWrapper;
use axum::{Json, extract::State, http::StatusCode};
use db_schema::{
    models::{data::DataModel, servers::ServerModel},
    schema::*,
};
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
    pub licensed: Option<bool>,
    pub white_list: Option<bool>,
    pub checked: Option<bool>,
    pub auth_me: Option<bool>,
    pub crashed: Option<bool>,
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

    let pagination_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
        if let Some(ref offset_ip) = body.offset_ip {
            let id = servers::table
                .filter(servers::ip.eq(offset_ip))
                .select(servers::id)
                .first::<i32>(&mut conn)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Box::new(servers::id.lt(id))
        } else {
            let id = servers::table
                .select(max(servers::id))
                .first::<Option<i32>>(&mut conn)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .unwrap_or(i32::MAX);
            Box::new(servers::id.le(id))
        };

    let license_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.licensed {
        Some(license) => Box::new(servers::license.eq(license)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let white_list_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.white_list
    {
        Some(white_list) => Box::new(servers::white_list.assume_not_null().eq(white_list)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let checked_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.checked {
        Some(checked) => Box::new(servers::checked.assume_not_null().eq(checked)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let auth_me_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.auth_me {
        Some(auth_me) => Box::new(servers::auth_me.assume_not_null().eq(auth_me)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let crashed_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.crashed {
        Some(crashed) => Box::new(servers::crashed.assume_not_null().eq(crashed)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let results = servers::table
        .inner_join(data::table.on(data::server_id.eq(servers::id)))
        .filter(pagination_filter)
        .filter(license_filter)
        .filter(white_list_filter)
        .filter(checked_filter)
        .filter(auth_me_filter)
        .filter(crashed_filter)
        .order((servers::id.desc(), data::id.desc()))
        .distinct_on(servers::id)
        .select((ServerModel::as_select(), DataModel::as_select()))
        .limit(body.limit)
        .load::<(ServerModel, DataModel)>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results.into_iter().map(Into::into).collect()))
}
