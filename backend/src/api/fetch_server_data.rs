use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use db_schema::{models::data::DataModel, schema::data};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::database::DatabaseWrapper;

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

impl From<DataModel> for ServerDataResponse {
    fn from(value: DataModel) -> Self {
        ServerDataResponse {
            server_id: value.server_id,
            online: value.online,
            max: value.max,
            players: value
                .players
                .as_array()
                .unwrap()
                .into_iter()
                .map(|t| t.as_str().unwrap().to_string())
                .collect(),
            timestamp: value.timestamp,
        }
    }
}

pub async fn fetch_server_data(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerDataRequest>,
) -> Result<Json<Vec<ServerDataResponse>>, StatusCode> {
    let mut conn = db.pool.get().await.unwrap();

    let results: Vec<DataModel> = data::table
        .filter(data::server_id.eq(body.server_id))
        .order(data::id.desc())
        .limit(body.limit)
        .load(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results.into_iter().map(Into::into).collect()))
}
