use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::scan_targets::TargetInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use crate::{api::add_target::AddAddrRequest, database::DatabaseWrapper, error::AppError};

pub async fn add_addrs(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<Vec<AddAddrRequest>>,
) -> Result<StatusCode, AppError> {
    let new_targets: Vec<TargetInsert> = body
        .iter()
        .map(|t| t.to_target_insert())
        .collect::<Result<_, _>>()?;

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in add_ips", e))?;

    for chunk in new_targets.chunks(1000) {
        insert_into(schema::scan_targets::table)
            .values(chunk)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::db("Failed to bulk-insert IPs", e))?;
    }

    Ok(StatusCode::OK)
}
