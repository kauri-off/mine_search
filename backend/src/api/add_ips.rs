use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::ip::IpInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use crate::{api::add_ip::AddIpRequest, database::DatabaseWrapper};

pub async fn add_ips(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<Vec<AddIpRequest>>,
) -> Result<StatusCode, StatusCode> {
    let new_ips: Vec<IpInsert> = body
        .iter()
        .map(|t| IpInsert {
            ip: &t.ip,
            port: 25565,
        })
        .collect();

    let mut conn = db.pool.get().await.unwrap();

    for chunk in new_ips.chunks(1000) {
        insert_into(schema::ips::table)
            .values(chunk)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("DB insert failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(StatusCode::OK)
}
