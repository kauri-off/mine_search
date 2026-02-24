use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::server_ping::ServerPingInsert, schema::server_ping};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct PingServerRequest {
    pub server_id: i32,
    pub with_connection: bool,
}

pub async fn ping_server(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<PingServerRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in update_server", e))?;

    let ping_insert = ServerPingInsert {
        server_id: body.server_id,
        with_connection: body.with_connection,
    };

    insert_into(server_ping::table)
        .values(&ping_insert)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to insert ping"), e))?;

    Ok(StatusCode::OK)
}
