use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::schema::servers;
use diesel::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize)]
pub struct Body {
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

impl From<Body> for Options {
    fn from(value: Body) -> Self {
        Options {
            checked: value.checked,
            spoofable: value.spoofable,
            crashed: value.crashed,
        }
    }
}

pub async fn update_server(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<Body>,
) -> Result<StatusCode, StatusCode> {
    update(servers::table)
        .filter(servers::ip.eq(body.server_ip.clone()))
        .set::<Options>(body.into())
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
