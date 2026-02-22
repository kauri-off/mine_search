use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Utc;
use db_schema::{models::data::DataModel, schema::data};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ServerDataRequest {
    pub server_id: i32,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ServerDataResponse {
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
    pub players: Vec<String>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl TryFrom<DataModel> for ServerDataResponse {
    type Error = AppError;

    fn try_from(value: DataModel) -> Result<Self, Self::Error> {
        let players = value
            .players
            .as_array()
            .ok_or_else(|| {
                AppError::internal(
                    "Malformed players JSON: expected an array",
                    format!("got: {}", value.players),
                )
            })?
            .iter()
            .map(|t| {
                t.as_str()
                    .ok_or_else(|| {
                        AppError::internal(
                            "Malformed players JSON: element is not a string",
                            format!("got: {t}"),
                        )
                    })
                    .map(str::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ServerDataResponse {
            server_id: value.server_id,
            online: value.online,
            max: value.max,
            players,
            timestamp: value.timestamp,
        })
    }
}

pub async fn fetch_server_data(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerDataRequest>,
) -> Result<Json<Vec<ServerDataResponse>>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_server_data", e))?;

    let results: Vec<DataModel> = data::table
        .filter(data::server_id.eq(body.server_id))
        .order(data::id.desc())
        .limit(body.limit)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to load server data", e))?;

    let responses = results
        .into_iter()
        .map(ServerDataResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(responses))
}
