use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema::servers;
use diesel::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdateServerRequest {
    pub server_ip: String,
    pub checked: Option<bool>,
    pub spoofable: Option<bool>,
    pub crashed: Option<bool>,
}

#[derive(AsChangeset)]
#[diesel(table_name = servers)]
pub struct Options {
    pub checked: Option<bool>,
    pub spoofable: Option<bool>,
    pub crashed: Option<bool>,
}

impl From<UpdateServerRequest> for Options {
    fn from(value: UpdateServerRequest) -> Self {
        Options {
            checked: value.checked,
            spoofable: value.spoofable,
            crashed: value.crashed,
        }
    }
}

pub async fn update_server(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<UpdateServerRequest>,
) -> Result<StatusCode, AppError> {
    let server_ip = body.server_ip.clone();

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in update_server", e))?;

    update(servers::table)
        .filter(servers::ip.eq(&server_ip))
        .set::<Options>(body.into())
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to update server '{server_ip}'"), e))?;

    Ok(StatusCode::OK)
}
