use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::ip::IpInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct AddIpRequest {
    pub ip: String,
}

pub async fn add_ip(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<AddIpRequest>,
) -> Result<StatusCode, AppError> {
    let new_ip = IpInsert {
        ip: &body.ip,
        port: 25565,
    };

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in add_ip", e))?;

    insert_into(schema::ips::table)
        .values(new_ip)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to insert IP", e))?;

    Ok(StatusCode::OK)
}
