use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use db_schema::{
    models::{players::PlayerModel, servers::ServerModel},
    schema::servers::dsl::*,
};
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize)]
pub struct PlayersRequest {
    pub server_ip: String,
}

pub async fn get_players(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<PlayersRequest>,
) -> Result<Json<Vec<PlayerModel>>, StatusCode> {
    let mut conn = db.pool.get().await.unwrap();

    let server: ServerModel = servers
        .filter(ip.eq(body.server_ip))
        .select(ServerModel::as_select())
        .get_result(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let players = PlayerModel::belonging_to(&server)
        .select(PlayerModel::as_select())
        .get_results(&mut conn)
        .await
        .unwrap();

    Ok(Json(players))
}
