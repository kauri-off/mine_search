//! Renders a Minecraft MOTD/chat JSON object to HTML. Moved verbatim from the
//! old REST `fetch_server_info` handler. Output is HTML-escaped here; the
//! frontend additionally sanitizes with DOMPurify.

use db_schema::chat::{ChatComponentObject, ChatObject};
use serde_json::Value;

pub fn parse_html(value: Value) -> String {
    match serde_json::from_value::<ChatObject>(value) {
        Ok(t) => chat_object_to_html(&t),
        Err(_) => "<span></span>".to_string(),
    }
}

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
