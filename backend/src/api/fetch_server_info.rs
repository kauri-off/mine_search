use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use db_schema::chat::{ChatComponentObject, ChatObject};
use db_schema::models::{player_count_snapshots::SnapshotModel, servers::ServerModel};
use db_schema::schema::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ServerInfoRequest {
    pub ip: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ServerInfoResponse {
    pub id: i32,
    pub ip: String,
    pub online: i32,
    pub max: i32,
    pub version_name: String,
    pub protocol: i32,
    pub license: bool,
    pub disconnect_reason_html: Option<String>,
    pub updated: DateTime<Utc>,
    pub description_html: String,
    pub was_online: bool,
    pub is_checked: bool,
    pub is_spoofable: Option<bool>,
    pub is_crashed: bool,
    pub requires_mods: bool,
    pub favicon: Option<String>,
    pub ping: Option<i64>,
}

impl From<(ServerModel, SnapshotModel)> for ServerInfoResponse {
    fn from((server, data): (ServerModel, SnapshotModel)) -> Self {
        Self {
            id: server.id,
            ip: server.ip,
            online: data.players_online.into(),
            max: data.players_max.into(),
            version_name: server.version_name,
            protocol: server.protocol,
            license: server.is_online_mode,
            disconnect_reason_html: server.disconnect_reason.map(parse_html),
            updated: server.updated_at,
            description_html: parse_html(server.description),
            was_online: server.is_online,
            is_checked: server.is_checked,
            is_spoofable: server.is_spoofable,
            is_crashed: server.is_crashed,
            requires_mods: server.requires_mods,
            favicon: server.favicon,
            ping: server.ping,
        }
    }
}

pub async fn fetch_server_info(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerInfoRequest>,
) -> Result<Json<ServerInfoResponse>, AppError> {
    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in fetch_server_info", e))?;

    let (server, data) = servers::table
        .inner_join(
            player_count_snapshots::table.on(player_count_snapshots::server_id.eq(servers::id)),
        )
        .filter(servers::ip.eq(&body.ip))
        .order_by(player_count_snapshots::recorded_at.desc())
        .select((ServerModel::as_select(), SnapshotModel::as_select()))
        .first::<(ServerModel, SnapshotModel)>(&mut conn)
        .await
        .map_err(|e| AppError::db(format!("Server '{}' not found", body.ip), e))?;

    Ok(Json((server, data).into()))
}

fn parse_html(value: Value) -> String {
    match serde_json::from_value::<ChatObject>(value) {
        Ok(t) => chat_object_to_html(&t),
        Err(_) => "<span></span>".to_string(),
    }
}

/// Escapes HTML-significant characters so attacker-controlled MOTD text from scanned
/// servers cannot inject markup. The frontend also sanitizes with DOMPurify, but
/// escaping here keeps the output well-formed and removes the single-barrier risk.
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn chat_object_to_html(chat: &ChatObject) -> String {
    match chat {
        ChatObject::Object(component) => chat_component_object_to_html(component),
        ChatObject::Array(array) => array.iter().map(chat_object_to_html).collect(),
        ChatObject::JsonPrimitive(value) => {
            if value.is_string() {
                format!(
                    "<span style=\"color: white;\" >{}</span>",
                    html_escape(value.as_str().unwrap_or_default())
                )
            } else {
                html_escape(&value.to_string())
            }
        }
    }
}

fn chat_component_object_to_html(component: &ChatComponentObject) -> String {
    let mut html = String::new();

    if let Some(text) = &component.text {
        let text = html_escape(text).replace('\n', "<br>");
        let tag = "span";

        let mut style = String::new();
        if component.bold.unwrap_or(false) {
            style.push_str("font-weight: bold;");
        }
        if component.italic.unwrap_or(false) {
            style.push_str("font-style: italic;");
        }
        if component.underlined.unwrap_or(false) {
            style.push_str("text-decoration: underline;");
        }
        if component.strikethrough.unwrap_or(false) {
            style.push_str("text-decoration: line-through;");
        }
        if component.obfuscated.unwrap_or(false) {
            style.push_str("text-decoration: blink;");
        }
        if let Some(color) = &component.color {
            style.push_str(&format!("color: {}; ", html_escape(color)));
        } else {
            style.push_str("color: white; ");
        }

        if !style.is_empty() {
            html.push_str(&format!("<{} style=\"{}\">", tag, style));
        } else {
            html.push_str(&format!("<{}>", tag));
        }

        html.push_str(&text);
        html.push_str(&format!("</{}>", tag));
    }

    if let Some(extra) = &component.extra {
        for sub_object in extra {
            html.push_str(&chat_object_to_html(sub_object));
        }
    }

    html
}
