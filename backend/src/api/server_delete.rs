use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ServerDeleteRequest {
    pub id: i32,
}

pub async fn server_delete(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerDeleteRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in server_delete", e))?;

    diesel::delete(schema::servers::table)
        .filter(schema::servers::id.eq(body.id))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to delete server id={}", body.id), e))?;

    Ok(StatusCode::OK)
}
