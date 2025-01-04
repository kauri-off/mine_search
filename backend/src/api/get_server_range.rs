use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use db_schema::schema::servers::dsl::*;
use diesel::{
    query_dsl::methods::{LimitDsl, OffsetDsl, SelectDsl},
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseWrapper, ServerModel};

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub limit: i64,
    pub offset: i64,
}

pub async fn get_server_range(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<Vec<ServerModel>>, StatusCode> {
    let server_list = servers
        .limit(body.limit)
        .offset(body.offset)
        .select(ServerModel::as_select())
        .load(&mut db.pool.get().await.unwrap())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(server_list))
}
