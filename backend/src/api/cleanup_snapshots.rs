use std::sync::Arc;

use axum::{Json, extract::State};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Serialize, TS)]
#[ts(export)]
pub struct CleanupResponse {
    #[ts(type = "number")]
    pub deleted: i64,
}

pub async fn cleanup_snapshots(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<CleanupResponse>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in cleanup_snapshots", e))?;

    // Delete all but the latest 100 snapshots per server
    let deleted = diesel::sql_query(
        "DELETE FROM player_count_snapshots
         WHERE id IN (
             SELECT id FROM (
                 SELECT id,
                        ROW_NUMBER() OVER (PARTITION BY server_id ORDER BY recorded_at DESC) AS rn
                 FROM player_count_snapshots
             ) ranked
             WHERE rn > 100
         )",
    )
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::db("Failed to clean snapshots", e))?;

    Ok(Json(CleanupResponse {
        deleted: deleted as i64,
    }))
}
