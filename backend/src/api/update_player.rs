use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::players::PlayerUpdate, schema::players};
use diesel::{dsl::*, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{api::fetch_players_list::PlayerStatus, database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdatePlayerRequest {
    pub id: i32,
    pub status: PlayerStatus,
}

pub async fn update_player(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in update_server", e))?;

    let player_update = PlayerUpdate {
        status: &body.status.into(),
    };

    update(players::table)
        .filter(players::id.eq(&body.id))
        .set(&player_update)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Failed to update player"), e))?;

    Ok(StatusCode::OK)
}
