use std::sync::Arc;

use axum::{Json, extract::State};
use db_schema::schema::servers;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatsResponse {
    #[ts(type = "number")]
    pub total_servers: i64,
    #[ts(type = "number")]
    pub cracked_servers: i64,
}

pub async fn fetch_stats(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<StatsResponse>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_stats", e))?;

    let total_servers = servers::dsl::servers
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count total servers", e))?;

    let cracked_servers = servers::dsl::servers
        .filter(servers::dsl::license.eq(false))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count cracked servers", e))?;

    Ok(Json(StatsResponse {
        total_servers,
        cracked_servers,
    }))
}
