use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDateTime;
use db_schema::{models::servers::ServerModel, schema::servers::dsl::*};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::DatabaseWrapper;

#[derive(Serialize, Deserialize)]
pub struct ServerRequest {
    pub ip: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerResponse {
    pub ip: String,
    pub online: i32,
    pub max: i32,
    pub version_name: String,
    pub protocol: i32,
    pub license: bool,
    pub white_list: Option<bool>,
    pub last_seen: NaiveDateTime,
    pub description: Value,
    pub description_html: String,
}

impl From<ServerModel> for ServerResponse {
    fn from(value: ServerModel) -> Self {
        ServerResponse {
            ip: value.ip,
            online: value.online,
            max: value.max,
            version_name: value.version_name,
            protocol: value.protocol,
            license: value.license,
            white_list: value.white_list,
            last_seen: value.last_seen,
            description: value.description.clone(),
            description_html: parse_html(value.description),
        }
    }
}

fn parse_html(value: Value) -> String {
    let payload: Description = serde_json::from_value(value).unwrap();

    match payload.payload {
        Some(t) => chat_object_to_html(&t),
        None => "<span></span>".to_string(),
    }
}

pub async fn fetch_server_info(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<ServerRequest>,
) -> Result<Json<ServerResponse>, StatusCode> {
    let server = servers
        .filter(ip.eq(body.ip))
        .select(ServerModel::as_select())
        .first(&mut db.pool.get().await.unwrap())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(server.into()))
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
        let tag = "span".to_string(); // Default to a <span> tag

        // Apply styles based on the options provided
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
            style.push_str("text-decoration: blink;"); // Obfuscation is tricky, so this is a placeholder
        }
        if let Some(color) = &component.color {
            style.push_str(&format!("color: {}; ", color));
        } else {
            style.push_str("color: white; ");
        }

        // If there are any styles, apply them to the tag
        if !style.is_empty() {
            html.push_str(&format!("<{} style=\"{}\">", tag, style));
        } else {
            html.push_str(&format!("<{}>", tag));
        }

        // Add the text content
        html.push_str(&text);

        // Close the tag
        html.push_str(&format!("</{}>", tag));
    }

    // Handle extra components (nested objects)
    if let Some(extra) = &component.extra {
        for sub_object in extra {
            html.push_str(&chat_object_to_html(sub_object));
        }
    }

    html
}

#[derive(Serialize, Deserialize)]
struct Description {
    payload: Option<ChatObject>,
}

/// Represents a chat object (the MOTD is sent as a chat object).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    /// An individual chat object
    Object(ChatComponentObject),

    /// Vector of multiple chat objects
    Array(Vec<ChatObject>),

    /// Unknown data - raw JSON
    JsonPrimitive(serde_json::Value),
}

/// A piece of a `ChatObject`
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatComponentObject {
    /// Text of the chat message
    pub text: Option<String>,

    /// Translation key if the message needs to pull from the language file.
    /// See [wiki.vg](https://wiki.vg/Chat#Translation_component)
    pub translate: Option<String>,

    /// Displays the keybind for the specified key, or the string itself if unknown.
    pub keybind: Option<String>,

    /// Should the text be rendered **bold**?
    pub bold: Option<bool>,

    /// Should the text be rendered *italic*?
    pub italic: Option<bool>,

    /// Should the text be rendered __underlined__?
    pub underlined: Option<bool>,

    /// Should the text be rendered as ~~strikethrough~~
    pub strikethrough: Option<bool>,

    /// Should the text be rendered as obfuscated?
    /// Switching randomly between characters of the same width
    pub obfuscated: Option<bool>,

    /// The font to use to render, comes in three options:
    /// * `minecraft:uniform` - Unicode font
    /// * `minecraft:alt` - enchanting table font
    /// * `minecraft:default` - font based on resource pack (1.16+)
    ///
    /// Any other value can be ignored
    pub font: Option<String>,

    /// The color to display the chat item in.
    /// Can be a [chat color](https://wiki.vg/Chat#Colors),
    /// [format code](https://wiki.vg/Chat#Styles),
    /// or any valid web color
    pub color: Option<String>,

    /// Text to insert into the chat box when shift-clicking this component
    pub insertion: Option<String>,

    /// Defines an event that occurs when this chat item is clicked
    #[serde(rename = "clickEvent")]
    pub click_event: Option<ChatClickEvent>,

    /// Defines an event that occurs when this chat item is hovered on
    #[serde(rename = "hoverEvent")]
    pub hover_event: Option<ChatHoverEvent>,

    /// Sibling components to this chat item.
    /// If present, will not be empty
    pub extra: Option<Vec<ChatObject>>,
}

/// `ClickEvent` data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatClickEvent {
    // These are not renamed on purpose. (server returns them in snake_case)
    /// Opens the URL in the user's default browser. Protocol must be `http` or `https`
    pub open_url: Option<String>,

    /// Runs the command.
    /// Simply causes the user to say the string in chat -
    /// so only has command effect if it starts with /
    ///
    /// Irrelevant for motd purposes.
    pub run_command: Option<String>,

    /// Replaces the content of the user's chat box with the given text.
    ///
    /// Irrelevant for motd purposes.
    pub suggest_command: Option<String>,

    /// Copies the given text into the client's clipboard.
    pub copy_to_clipboard: Option<String>,
}

/// `HoverEvent` data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHoverEvent {
    // These are not renamed on purpose. (server returns them in snake_case)
    /// Text to show when the item is hovered over
    pub show_text: Option<Box<ChatObject>>,

    /// Same as `show_text`, but for servers < 1.16
    pub value: Option<Box<ChatObject>>,

    /// Displays the item of the given NBT
    pub show_item: Option<String>,

    /// Displays information about the entity with the given NBT
    pub show_entity: Option<String>,
}
