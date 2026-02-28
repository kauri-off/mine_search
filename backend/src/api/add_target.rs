use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::scan_targets::TargetInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct AddAddrRequest {
    pub addr: String,
    pub quick: bool,
}

impl AddAddrRequest {
    pub fn to_target_insert<'a>(&'a self) -> Result<TargetInsert<'a>, AppError> {
        // Check if addr is [ip]:[port]
        let (ip, port) = if let Some((ip, port_str)) = self.addr.rsplit_once(':') {
            let port = port_str
                .parse::<i32>()
                .map_err(|_| AppError::bad_request(format!("Invalid port: {port_str}")))?;
            (ip, port)
        } else {
            (self.addr.as_str(), 25565)
        };

        Ok(TargetInsert {
            ip,
            port,
            quick: self.quick,
        })
    }
}

pub async fn add_target(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<AddAddrRequest>,
) -> Result<StatusCode, AppError> {
    let target = body.to_target_insert()?;

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in add_target", e))?;

    insert_into(schema::scan_targets::table)
        .values(&target)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to insert target", e))?;

    Ok(StatusCode::OK)
}
