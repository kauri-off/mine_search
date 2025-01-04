use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use db_schema::schema::servers::dsl::*;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseWrapper, ServerModel};

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub ip: String,
}

pub async fn get_server(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<ServerModel>, StatusCode> {
    let server = servers
        .filter(ip.eq(body.ip))
        .select(ServerModel::as_select())
        .first(&mut db.pool.get().await.unwrap())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(server))
}
