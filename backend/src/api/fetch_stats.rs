use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema::servers;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatsResponse {
    pub total_servers: i32,
    pub cracked_servers: i32,
}

pub async fn fetch_stats(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<StatsResponse>, StatusCode> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_servers = servers::dsl::servers
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as i32;

    let cracked_servers = servers::dsl::servers
        .filter(servers::dsl::license.eq(false))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as i32;

    Ok(Json(StatsResponse {
        total_servers,
        cracked_servers,
    }))
}
