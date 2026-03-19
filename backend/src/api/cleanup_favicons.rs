use std::sync::Arc;

use axum::{Json, extract::State};
use db_schema::schema::servers;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;

use crate::{
    api::cleanup_snapshots::CleanupResponse, database::DatabaseWrapper, error::AppError,
};

pub async fn cleanup_favicons(
    State(db): State<Arc<DatabaseWrapper>>,
) -> Result<Json<CleanupResponse>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in cleanup_favicons", e))?;

    let deleted = diesel::update(servers::table)
        .filter(servers::favicon.is_not_null())
        .set(servers::favicon.eq(None::<String>))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to clear favicons", e))?;

    // Run VACUUM in background so the HTTP response returns immediately
    let db2 = Arc::clone(&db);
    tokio::spawn(async move {
        if let Ok(mut conn) = db2.pool.get().await {
            let _ = diesel::sql_query("VACUUM servers")
                .execute(&mut conn)
                .await;
        }
    });

    Ok(Json(CleanupResponse {
        deleted: deleted as i64,
    }))
}
