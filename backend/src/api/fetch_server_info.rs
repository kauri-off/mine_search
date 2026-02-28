use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
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
    pub is_forge: bool,
    pub favicon: Option<String>,
}

impl From<(ServerModel, SnapshotModel)> for ServerInfoResponse {
    fn from((server, data): (ServerModel, SnapshotModel)) -> Self {
        Self {
            id: server.id,
            ip: server.ip,
            online: data.players_online,
            max: data.players_max,
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
            is_forge: server.is_forge,
            favicon: server.favicon,
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
        .order_by(player_count_snapshots::id.desc())
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

fn chat_object_to_html(chat: &ChatObject) -> String {
    match chat {
        ChatObject::Object(component) => chat_component_object_to_html(component),
        ChatObject::Array(array) => array.iter().map(|obj| chat_object_to_html(obj)).collect(),
        ChatObject::JsonPrimitive(value) => {
            if value.is_string() {
                format!(
                    "<span style=\"color: white;\" >{}</span>",
                    value.as_str().unwrap()
                )
            } else {
                value.to_string()
            }
        }
    }
}

fn chat_component_object_to_html(component: &ChatComponentObject) -> String {
    let mut html = String::new();

    if let Some(text) = &component.text {
        let text = text.replace("\n", "<br>");
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
            style.push_str(&format!("color: {}; ", color));
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    Object(ChatComponentObject),
    Array(Vec<ChatObject>),
    JsonPrimitive(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatComponentObject {
    pub text: Option<String>,
    pub translate: Option<String>,
    pub keybind: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub font: Option<String>,
    pub color: Option<String>,
    pub insertion: Option<String>,
    #[serde(rename = "clickEvent")]
    pub click_event: Option<ChatClickEvent>,
    #[serde(rename = "hoverEvent")]
    pub hover_event: Option<ChatHoverEvent>,
    pub extra: Option<Vec<ChatObject>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatClickEvent {
    pub open_url: Option<String>,
    pub run_command: Option<String>,
    pub suggest_command: Option<String>,
    pub copy_to_clipboard: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHoverEvent {
    pub show_text: Option<Box<ChatObject>>,
    pub value: Option<Box<ChatObject>>,
    pub show_item: Option<String>,
    pub show_entity: Option<String>,
}
