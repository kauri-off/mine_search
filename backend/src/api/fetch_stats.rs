use std::sync::Arc;

use axum::{Json, extract::State};
use db_schema::{
    models::players::PlayerStatus,
    schema::{players, servers},
};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Double, Nullable};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Serialize, TS)]
#[ts(export)]
pub struct VersionStat {
    pub version: String,
    #[ts(type = "number")]
    pub count: i64,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct StatsResponse {
    #[ts(type = "number")]
    pub total_servers: i64,
    #[ts(type = "number")]
    pub cracked_servers: i64,
    #[ts(type = "number")]
    pub online_servers: i64,
    #[ts(type = "number")]
    pub crashed_servers: i64,
    #[ts(type = "number")]
    pub forge_servers: i64,
    #[ts(type = "number")]
    pub spoofable_servers: i64,
    #[ts(type = "number")]
    pub total_players: i64,
    #[ts(type = "number")]
    pub admin_players: i64,
    pub avg_ping: Option<f64>,
    pub version_distribution: Vec<VersionStat>,
    pub db_size_mb: f64,
    pub favicon_size_mb: f64,
}

pub async fn fetch_stats(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<StatsResponse>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_stats", e))?;

    let total_servers = servers::table
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count total servers", e))?;

    let cracked_servers = servers::table
        .filter(servers::is_online_mode.eq(false))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count cracked servers", e))?;

    let online_servers = servers::table
        .filter(servers::is_online.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count online servers", e))?;

    let crashed_servers = servers::table
        .filter(servers::is_crashed.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count crashed servers", e))?;

    let forge_servers = servers::table
        .filter(servers::is_forge.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count forge servers", e))?;

    let spoofable_servers = servers::table
        .filter(servers::is_spoofable.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count spoofable servers", e))?;

    let total_players = players::table
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count total players", e))?;

    let admin_players = players::table
        .filter(players::status.eq(PlayerStatus::Admin))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to count admin players", e))?;

    let avg_ping = servers::table
        .select(diesel::dsl::sql::<Nullable<Double>>("AVG(ping)::float8"))
        .get_result::<Option<f64>>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to compute avg ping", e))?;

    let version_rows = servers::table
        .group_by(servers::version_name)
        .select((servers::version_name, diesel::dsl::count_star()))
        .order(diesel::dsl::count_star().desc())
        .limit(10)
        .load::<(String, i64)>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to load version distribution", e))?;

    let version_distribution = version_rows
        .into_iter()
        .map(|(version, count)| VersionStat { version, count })
        .collect();

    let db_size_bytes = diesel::dsl::sql::<BigInt>("SELECT pg_database_size(current_database())")
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to get DB size", e))?;

    let favicon_size_bytes = diesel::dsl::sql::<BigInt>(
        "SELECT COALESCE(SUM(octet_length(favicon)), 0) FROM servers WHERE favicon IS NOT NULL",
    )
    .get_result::<i64>(&mut conn)
    .await
    .map_err(|e| AppError::db("Failed to get favicon size", e))?;

    Ok(Json(StatsResponse {
        total_servers,
        cracked_servers,
        online_servers,
        crashed_servers,
        forge_servers,
        spoofable_servers,
        total_players,
        admin_players,
        avg_ping,
        version_distribution,
        db_size_mb: db_size_bytes as f64 / 1_048_576.0,
        favicon_size_mb: favicon_size_bytes as f64 / 1_048_576.0,
    }))
}
