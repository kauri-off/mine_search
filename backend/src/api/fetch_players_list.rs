use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use db_schema::{
    models::{self, players::PlayerModel},
    schema::players,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct PlayerListRequest {
    pub server_id: i32,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PlayerStatus {
    None,
    Regular,
    Admin,
}

impl From<models::players::PlayerStatus> for PlayerStatus {
    fn from(s: models::players::PlayerStatus) -> Self {
        match s {
            models::players::PlayerStatus::None => PlayerStatus::None,
            models::players::PlayerStatus::Regular => PlayerStatus::Regular,
            models::players::PlayerStatus::Admin => PlayerStatus::Admin,
        }
    }
}

impl From<PlayerStatus> for models::players::PlayerStatus {
    fn from(s: PlayerStatus) -> Self {
        match s {
            PlayerStatus::None => models::players::PlayerStatus::None,
            PlayerStatus::Regular => models::players::PlayerStatus::Regular,
            PlayerStatus::Admin => models::players::PlayerStatus::Admin,
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PlayerResponse {
    pub id: i32,
    pub server_id: i32,
    pub name: String,
    pub status: PlayerStatus,
    #[ts(type = "string")]
    pub last_seen_at: DateTime<Utc>,
}

impl From<PlayerModel> for PlayerResponse {
    fn from(value: PlayerModel) -> Self {
        PlayerResponse {
            id: value.id,
            server_id: value.server_id,
            name: value.name,
            status: value.status.into(),
            last_seen_at: value.last_seen_at,
        }
    }
}

pub async fn fetch_players_list(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<PlayerListRequest>,
) -> Result<Json<Vec<PlayerResponse>>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_server_data", e))?;

    let result = players::table
        .filter(players::server_id.eq(body.server_id))
        .select(PlayerModel::as_select())
        .order(players::last_seen_at.desc())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to load server data", e))?;

    Ok(Json(result.into_iter().map(Into::into).collect()))
}
