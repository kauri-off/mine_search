use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use db_schema::{models::data::DataModel, schema::data};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub server_id: i32,
    pub limit: i64,
}

#[derive(Serialize)]
pub struct DataResponse {
    pub server_id: i32,
    pub online: i32,
    pub max: i32,
    pub players: serde_json::Value, // Vec<String>
    pub timestamp: chrono::DateTime<Utc>,
}

impl From<DataModel> for DataResponse {
    fn from(value: DataModel) -> Self {
        DataResponse {
            server_id: value.server_id,
            online: value.online,
            max: value.max,
            players: value.players,
            timestamp: value.timestamp,
        }
    }
}

pub async fn fetch_server_data(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<Vec<DataResponse>>, StatusCode> {
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
