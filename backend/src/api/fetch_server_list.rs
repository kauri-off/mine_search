use super::fetch_server_info::ServerInfoResponse;
use crate::{database::DatabaseWrapper, error::AppError};
use axum::{Json, extract::State};
use db_schema::{
    models::{data::DataModel, servers::ServerModel},
    schema::*,
};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{dsl::sql, pg::Pg};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ServerListRequest {
    #[ts(type = "number")]
    pub limit: i64,
    pub offset_id: Option<i32>,
    pub licensed: Option<bool>,
    pub checked: Option<bool>,
    pub spoofable: Option<bool>,
    pub crashed: Option<bool>,
    pub has_players: Option<bool>,
    pub online: Option<bool>,
    pub forge: Option<bool>,
}

pub async fn fetch_server_list(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerListRequest>,
) -> Result<Json<Vec<ServerInfoResponse>>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_server_list", e))?;

    let pagination_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
        if let Some(id) = body.offset_id {
            Box::new(servers::id.lt(id))
        } else {
            Box::new(sql::<Bool>("TRUE"))
        };

    let license_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.licensed {
        Some(license) => Box::new(servers::license.eq(license)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let checked_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.checked {
        Some(checked) => Box::new(servers::checked.assume_not_null().eq(checked)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let spoofable_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.spoofable {
        Some(spoofable) => Box::new(servers::spoofable.assume_not_null().eq(spoofable)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let crashed_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.crashed {
        Some(crashed) => Box::new(servers::crashed.assume_not_null().eq(crashed)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let has_players_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
        match body.has_players {
            Some(true) => Box::new(diesel::dsl::exists(
                players::table.filter(players::server_id.eq(servers::id)),
            )),
            Some(false) => Box::new(diesel::dsl::not(diesel::dsl::exists(
                players::table.filter(players::server_id.eq(servers::id)),
            ))),
            None => Box::new(sql::<Bool>("TRUE")),
        };

    let online_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.online {
        Some(online) => Box::new(servers::was_online.eq(online)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let forge_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.forge {
        Some(true) => Box::new(sql::<Bool>("disconnect_reason::text ILIKE '%forge%'")),
        Some(false) => Box::new(sql::<Bool>(
            "disconnect_reason IS NULL OR disconnect_reason::text NOT ILIKE '%forge%'",
        )),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let results = servers::table
        .inner_join(data::table.on(data::server_id.eq(servers::id)))
        .filter(pagination_filter)
        .filter(license_filter)
        .filter(checked_filter)
        .filter(spoofable_filter)
        .filter(crashed_filter)
        .filter(has_players_filter)
        .filter(online_filter)
        .filter(forge_filter)
        .order((servers::id.desc(), data::id.desc()))
        .distinct_on(servers::id)
        .select((ServerModel::as_select(), DataModel::as_select()))
        .limit(body.limit)
        .load::<(ServerModel, DataModel)>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to load server list", e))?;

    Ok(Json(results.into_iter().map(Into::into).collect()))
}
