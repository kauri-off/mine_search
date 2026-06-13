//! Minecraft chat-component model (server MOTD / disconnect reason).
//!
//! Shared between the worker (plaintext extraction via [`ChatObject::get_motd`]) and
//! the backend (HTML rendering). The data model lives here; renderers live with their
//! respective consumer so this crate stays presentation-free.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    Object(ChatComponentObject),
    Array(Vec<ChatObject>),
    JsonPrimitive(Value),
}

impl ChatObject {
    /// Flattens the component tree into plain text, discarding all formatting.
    pub fn get_motd(&self) -> String {
        match self {
            ChatObject::Object(component) => {
                let mut result = String::new();
                if let Some(text) = &component.text {
                    result += text.as_str();
                }
                if let Some(extra) = &component.extra {
                    for object in extra {
                        result += &object.get_motd();
                    }
                }
                result
            }
            ChatObject::Array(vec) => {
                let mut result = String::new();
                for object in vec {
                    result += &object.get_motd();
                }
                result
            }
            ChatObject::JsonPrimitive(value) => {
                value.as_str().map(|s| s.to_string()).unwrap_or_default()
            }
        }
    }
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
