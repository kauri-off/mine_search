use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::database::DatabaseWrapper;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ServerDeleteRequest {
    pub id: i32,
}

pub async fn server_delete(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerDeleteRequest>,
) -> Result<StatusCode, StatusCode> {
    diesel::delete(schema::servers::table)
        .filter(schema::servers::id.eq(body.id))
        .execute(
            &mut db
                .pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        )
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
