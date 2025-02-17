use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use db_schema::{models::servers::ServerModel, schema::servers::dsl::*};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseWrapper;

use super::get_server::ServerResponse;

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub limit: i64,
    pub offset_ip: Option<String>,
    pub license: Option<bool>,
}

pub async fn get_server_range(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<Vec<ServerResponse>>, StatusCode> {
    let mut conn = db.pool.get().await.unwrap();

    let server: ServerModel = if let Some(offset_ip) = body.offset_ip {
        servers
            .filter(ip.eq(offset_ip))
            .select(ServerModel::as_select())
            .get_result(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        servers
            .order(id.desc())
            .select(ServerModel::as_select())
            .get_result(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let server_list: Vec<ServerModel> = if let Some(l) = body.license {
        servers
            .filter(id.lt(server.id))
            .filter(license.eq(l))
            .limit(body.limit)
            .order(id.desc())
            .select(ServerModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        servers
            .filter(id.lt(server.id))
            .limit(body.limit)
            .order(id.desc())
            .select(ServerModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(server_list.into_iter().map(|t| t.into()).collect()))
}
