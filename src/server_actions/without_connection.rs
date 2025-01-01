use std::io::{self, ErrorKind};

use minecraft_protocol::types::var_int::VarInt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;

use crate::{
    conn_wrapper::ConnectionWrapper,
    packets::{Handshake, StatusRequest, StatusResponse},
};

#[derive(Deserialize, Debug)]
pub struct Status {
    pub players: Players,
    pub version: Version,
    pub description: ChatObject,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    Object(ChatComponentObject),
    Array(Vec<ChatObject>),
    JsonPrimitive(Value),
}

impl ChatObject {
    pub fn get_motd(&self) -> Option<String> {
        match self {
            ChatObject::Object(chat_component_object) => {
                let mut result = String::new();

                if let Some(text) = &chat_component_object.text {
                    result += text.as_str();
                }

                if let Some(extra) = &chat_component_object.extra {
                    for object in extra {
                        result += &object.get_motd().unwrap_or("".to_string());
                    }
                }

                Some(result)
            }
            ChatObject::Array(vec) => {
                let mut result = String::new();

                for object in vec {
                    result += &object.get_motd().unwrap_or("".to_string());
                }

                Some(result)
            }
            ChatObject::JsonPrimitive(value) => value.as_str().map(|s| s.to_string()),
        }
    }
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

#[derive(Deserialize, Debug)]
pub struct Players {
    pub online: i64,
    pub max: i64,
    pub sample: Option<Vec<Player>>,
}

#[derive(Deserialize, Debug)]
pub struct Player {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i64,
}

pub async fn get_status(ip: &str, port: u16) -> io::Result<Status> {
    let mut conn = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    conn.write_packet(Handshake {
        protocol: VarInt(765),
        server_address: ip.to_string(),
        server_port: port,
        next_state: VarInt(1),
    })
    .await?;

    conn.write_packet(StatusRequest {}).await?;

    let response: StatusResponse = conn.read_packet().await?;

    let value: Status =
        serde_json::from_str(&response.response).map_err(|_| ErrorKind::InvalidData)?;
    Ok(value)
}
