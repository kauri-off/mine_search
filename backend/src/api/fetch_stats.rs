use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use db_schema::schema::{players, servers};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize)]
pub struct StatsReturn {
    pub total_servers: i64,
    pub cracked_servers: i64,
    pub players: i64,
}

pub async fn fetch_stats(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<StatsReturn>, StatusCode> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_servers: i64 = servers::dsl::servers
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cracked_servers: i64 = servers::dsl::servers
        .filter(servers::dsl::license.eq(false))
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let players: i64 = players::dsl::players
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StatsReturn {
        total_servers,
        cracked_servers,
        players,
    }))
}
