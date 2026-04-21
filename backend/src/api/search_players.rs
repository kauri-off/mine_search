use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use db_schema::{
    models::{players::PlayerModel, servers::ServerModelMini},
    schema::{players, servers},
};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{dsl::sql, pg::Pg};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{api::fetch_players_list::PlayerStatus, database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct PlayerSearchRequest {
    #[ts(type = "number")]
    pub limit: i64,
    pub offset_id: Option<i32>,
    pub name_contains: Option<String>,
    pub status: Option<PlayerStatus>,
    pub licensed: Option<bool>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PlayerSearchResponse {
    pub id: i32,
    pub server_id: i32,
    pub server_ip: String,
    pub name: String,
    pub status: PlayerStatus,
    #[ts(type = "string")]
    pub last_seen_at: DateTime<Utc>,
}

pub async fn search_players(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<PlayerSearchRequest>,
) -> Result<Json<Vec<PlayerSearchResponse>>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in search_players", e))?;

    let pagination_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
        if let Some(id) = body.offset_id {
            Box::new(players::id.lt(id))
        } else {
            Box::new(sql::<Bool>("TRUE"))
        };

    let name_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.name_contains {
        Some(ref s) if !s.is_empty() => Box::new(players::name.ilike(format!("%{}%", s))),
        _ => Box::new(sql::<Bool>("TRUE")),
    };

    use db_schema::models::players::PlayerStatus as DbPlayerStatus;
    let status_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.status {
        Some(PlayerStatus::None) => Box::new(players::status.eq(DbPlayerStatus::None)),
        Some(PlayerStatus::Regular) => Box::new(players::status.eq(DbPlayerStatus::Regular)),
        Some(PlayerStatus::Admin) => Box::new(players::status.eq(DbPlayerStatus::Admin)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let licensed_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.licensed {
        Some(v) => Box::new(servers::is_online_mode.eq(v)),
        None => Box::new(sql::<Bool>("TRUE")),
    };

    let results = players::table
        .inner_join(servers::table.on(servers::id.eq(players::server_id)))
        .filter(pagination_filter)
        .filter(name_filter)
        .filter(status_filter)
        .filter(licensed_filter)
        .order(players::id.desc())
        .select((PlayerModel::as_select(), ServerModelMini::as_select()))
        .limit(body.limit)
        .load::<(PlayerModel, ServerModelMini)>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to search players", e))?;

    Ok(Json(
        results
            .into_iter()
            .map(|(player, server)| PlayerSearchResponse {
                id: player.id,
                server_id: player.server_id,
                server_ip: server.ip,
                name: player.name,
                status: player.status.into(),
                last_seen_at: player.last_seen_at,
            })
            .collect(),
    ))
}
