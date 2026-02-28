use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Utc;
use db_schema::{models::player_count_snapshots::SnapshotModel, schema::player_count_snapshots};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ServerSnapshotsRequest {
    pub server_id: i32,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ServerSnapshotsResponse {
    pub server_id: i32,
    pub players_online: i32,
    pub players_max: i32,
    pub recorded_at: chrono::DateTime<Utc>,
}

impl TryFrom<SnapshotModel> for ServerSnapshotsResponse {
    type Error = AppError;

    fn try_from(value: SnapshotModel) -> Result<Self, Self::Error> {
        Ok(ServerSnapshotsResponse {
            server_id: value.server_id,
            players_online: value.players_online,
            players_max: value.players_max,
            recorded_at: value.recorded_at,
        })
    }
}

pub async fn fetch_server_snapshots(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerSnapshotsRequest>,
) -> Result<Json<Vec<ServerSnapshotsResponse>>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_server_data", e))?;

    let results: Vec<SnapshotModel> = player_count_snapshots::table
        .filter(player_count_snapshots::server_id.eq(body.server_id))
        .order(player_count_snapshots::id.desc())
        .limit(body.limit)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to load server data", e))?;

    let responses = results
        .into_iter()
        .map(ServerSnapshotsResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(responses))
}
