use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::ip::IpInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use crate::{api::add_ip::AddIpRequest, database::DatabaseWrapper, error::AppError};

pub async fn add_ips(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<Vec<AddIpRequest>>,
) -> Result<StatusCode, AppError> {
    let new_ips: Vec<IpInsert> = body
        .iter()
        .map(|t| IpInsert {
            ip: &t.ip,
            port: 25565,
        })
        .collect();

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in add_ips", e))?;

    for chunk in new_ips.chunks(1000) {
        insert_into(schema::ips::table)
            .values(chunk)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::db("Failed to bulk-insert IPs", e))?;
    }

    Ok(StatusCode::OK)
}
