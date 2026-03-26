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
pub struct DeletePlayerRequest {
    pub id: i32,
}

pub async fn delete_player(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<DeletePlayerRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in delete_player", e))?;

    diesel::delete(schema::players::table)
        .filter(schema::players::id.eq(body.id))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to delete player id={}", body.id), e))?;

    Ok(StatusCode::OK)
}
