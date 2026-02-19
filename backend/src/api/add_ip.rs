use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::ip::IpInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::database::DatabaseWrapper;

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct AddIpRequest {
    pub ip: String,
}

pub async fn add_ip(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<AddIpRequest>,
) -> Result<StatusCode, StatusCode> {
    let new_ip = IpInsert {
        ip: &body.ip,
        port: 25565,
    };

    insert_into(schema::ips::table)
        .values(new_ip)
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
