use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema::servers;
use diesel::dsl::update;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct OverwriteServerRequest {
    pub server_id: i32,
    pub port: Option<i32>,
    pub version_name: Option<String>,
    pub protocol: Option<i32>,
    pub is_online_mode: Option<bool>,
    pub is_forge: Option<bool>,
    pub is_online: Option<bool>,
    pub ping: Option<i64>,
    pub favicon: Option<String>,
    pub is_checked: Option<bool>,
    pub is_spoofable: Option<bool>,
    pub is_crashed: Option<bool>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = servers)]
struct ServerManualOverwrite {
    pub port: Option<i32>,
    pub version_name: Option<String>,
    pub protocol: Option<i32>,
    pub is_online_mode: Option<bool>,
    pub is_forge: Option<bool>,
    pub is_online: Option<bool>,
    pub ping: Option<i64>,
    pub favicon: Option<String>,
    pub is_checked: Option<bool>,
    pub is_spoofable: Option<bool>,
    pub is_crashed: Option<bool>,
}

impl From<OverwriteServerRequest> for ServerManualOverwrite {
    fn from(r: OverwriteServerRequest) -> Self {
        ServerManualOverwrite {
            port: r.port,
            version_name: r.version_name,
            protocol: r.protocol,
            is_online_mode: r.is_online_mode,
            is_forge: r.is_forge,
            is_online: r.is_online,
            ping: r.ping,
            favicon: r.favicon,
            is_checked: r.is_checked,
            is_spoofable: r.is_spoofable,
            is_crashed: r.is_crashed,
        }
    }
}

pub async fn overwrite_server(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<OverwriteServerRequest>,
) -> Result<StatusCode, AppError> {
    let server_id = body.server_id;

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in overwrite_server", e))?;

    update(servers::table)
        .filter(servers::id.eq(server_id))
        .set::<ServerManualOverwrite>(body.into())
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to overwrite server id={server_id}"), e))?;

    Ok(StatusCode::OK)
}
