//! Renders a Minecraft MOTD/chat JSON object to HTML. Moved verbatim from the
//! old REST `fetch_server_info` handler. Output is HTML-escaped here; the
//! frontend additionally sanitizes with DOMPurify.

use crate::chat::{ChatComponentObject, ChatObject};
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

/// Canonical Minecraft foreground palette. Accepts both the legacy single-char
/// codes (`0`-`9`, `a`-`f`) and the modern color names (`dark_red`, `gold`, …).
/// Returns `None` for anything else (e.g. an already-hex `#rrggbb` value or a
/// custom name) so callers can pass it through unchanged.
fn mc_color(name: &str) -> Option<&'static str> {
    let hex = match name {
        "0" | "black" => "#000000",
        "1" | "dark_blue" => "#0000AA",
        "2" | "dark_green" => "#00AA00",
        "3" | "dark_aqua" => "#00AAAA",
        "4" | "dark_red" => "#AA0000",
        "5" | "dark_purple" => "#AA00AA",
        "6" | "gold" => "#FFAA00",
        "7" | "gray" => "#AAAAAA",
        "8" | "dark_gray" => "#555555",
        "9" | "blue" => "#5555FF",
        "a" | "green" => "#55FF55",
        "b" | "aqua" => "#55FFFF",
        "c" | "red" => "#FF5555",
        "d" | "light_purple" => "#FF55FF",
        "e" | "yellow" => "#FFFF55",
        "f" | "white" => "#FFFFFF",
        _ => return None,
    };
    Some(hex)
}

/// Active legacy formatting state, emitted as inline CSS per styled run.
#[derive(Clone, Default)]
struct LegacyStyle {
    color: Option<&'static str>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    obfuscated: bool,
}

impl LegacyStyle {
    fn css(&self) -> String {
        let mut style = String::new();
        if self.bold {
            style.push_str("font-weight: bold;");
        }
        if self.italic {
            style.push_str("font-style: italic;");
        }
        if self.underline {
            style.push_str("text-decoration: underline;");
        }
        if self.strikethrough {
            style.push_str("text-decoration: line-through;");
        }
        if self.obfuscated {
            style.push_str("text-decoration: blink;");
        }
        // Default to white when no color code has been applied, matching the
        // look of the previous single-span renderer.
        style.push_str(&format!("color: {}; ", self.color.unwrap_or("#FFFFFF")));
        style
    }
}

/// Renders a string containing legacy `§` (section-sign) color/format codes into
/// styled HTML spans. A string with no codes yields a single white span,
/// identical to the previous behavior.
fn legacy_to_html(input: &str) -> String {
    let mut html = String::new();
    let mut style = LegacyStyle::default();
    let mut run = String::new();

    let flush = |html: &mut String, style: &LegacyStyle, run: &mut String| {
        if run.is_empty() {
            return;
        }
        let text = html_escape(run).replace('\n', "<br>");
        html.push_str(&format!("<span style=\"{}\">{}</span>", style.css(), text));
        run.clear();
    };

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\u{00a7}' {
            run.push(ch);
            continue;
        }
        // A `§` introduces a formatting code; consume the following char.
        let Some(code) = chars.next() else { break };
        // The style changes here, so emit everything accumulated so far first.
        flush(&mut html, &style, &mut run);
        match code.to_ascii_lowercase() {
            c @ ('0'..='9' | 'a'..='f') => {
                // A color code also resets active formatting (vanilla behavior).
                style = LegacyStyle {
                    color: mc_color(&c.to_string()),
                    ..LegacyStyle::default()
                };
            }
            'l' => style.bold = true,
            'o' => style.italic = true,
            'n' => style.underline = true,
            'm' => style.strikethrough = true,
            'k' => style.obfuscated = true,
            'r' => style = LegacyStyle::default(),
            _ => {} // unknown code: drop it
        }
    }
    flush(&mut html, &style, &mut run);

    if html.is_empty() {
        "<span style=\"color: #FFFFFF; \"></span>".to_string()
    } else {
        html
    }
}

fn chat_object_to_html(chat: &ChatObject) -> String {
    match chat {
        ChatObject::Object(component) => chat_component_object_to_html(component),
        ChatObject::Array(array) => array.iter().map(chat_object_to_html).collect(),
        ChatObject::JsonPrimitive(value) => {
            if value.is_string() {
                legacy_to_html(value.as_str().unwrap_or_default())
            } else {
                html_escape(&value.to_string())
            }
        }
    }
}

fn chat_component_object_to_html(component: &ChatComponentObject) -> String {
    let mut html = String::new();

    if let Some(text) = &component.text {
        // Some servers embed legacy `§` codes inside a structured component's
        // text; decode those into their own styled spans.
        if text.contains('\u{00a7}') {
            html.push_str(&legacy_to_html(text));
        } else {
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
            // Map Minecraft named colors (e.g. "dark_red") to real hex; pass
            // through anything else (a hex value or CSS keyword) unchanged.
            match component.color.as_deref() {
                Some(color) => {
                    let resolved = mc_color(color)
                        .map(str::to_string)
                        .unwrap_or_else(|| html_escape(color));
                    style.push_str(&format!("color: {}; ", resolved));
                }
                None => style.push_str("color: white; "),
            }

            html.push_str(&format!("<{} style=\"{}\">", tag, style));
            html.push_str(&text);
            html.push_str(&format!("</{}>", tag));
        }
    }

    if let Some(extra) = &component.extra {
        for sub_object in extra {
            html.push_str(&chat_object_to_html(sub_object));
        }
    }

    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_string_motd_renders_colored_spans() {
        // The real MOTD stored for 37.34.57.166 (row 24691): legacy `§` codes
        // in a bare JSON string.
        let value = json!("§cM§eI§dL§5A§2N§e'§6S §1P§cI§aK§eS§cE§6L§dR§cIJ§eK §c:) §f:) §1:)");
        let html = parse_html(value);

        // No literal section signs survive into the output.
        assert!(!html.contains('\u{00a7}'), "raw § leaked into HTML: {html}");
        // Codes are translated to real colors (§c -> red, §6 -> gold).
        assert!(html.contains("color: #FF5555"), "missing red span: {html}");
        assert!(html.contains("color: #FFAA00"), "missing gold span: {html}");
        // Letters survive.
        assert!(html.contains(">M</span>"));
    }

    #[test]
    fn plain_string_renders_single_white_span() {
        let html = parse_html(json!("Just a plain MOTD"));
        assert_eq!(
            html,
            "<span style=\"color: #FFFFFF; \">Just a plain MOTD</span>"
        );
    }

    #[test]
    fn named_color_maps_to_hex() {
        let value = json!({ "text": "hi", "color": "dark_red" });
        let html = parse_html(value);
        assert!(html.contains("color: #AA0000"), "got: {html}");
        assert!(!html.contains("dark_red"), "named color leaked: {html}");
    }

    #[test]
    fn unknown_color_passes_through_escaped() {
        let value = json!({ "text": "hi", "color": "#123abc" });
        let html = parse_html(value);
        assert!(html.contains("color: #123abc"), "got: {html}");
    }

    #[test]
    fn legacy_reset_and_format_codes() {
        // §l bold, then §r resets back to plain white.
        let html = legacy_to_html("§lbold§rplain");
        assert!(html.contains("font-weight: bold;"));
        // The "plain" run after §r has no bold.
        let plain = html.rsplit("plain").next().unwrap_or("");
        assert!(!plain.contains("font-weight: bold;"));
        assert!(html.contains(">bold</span>") && html.contains(">plain</span>"));
    }

    #[test]
    fn legacy_color_resets_active_format() {
        // A color code clears prior formatting (vanilla behavior): §a after §l
        // should not stay bold.
        let html = legacy_to_html("§lX§aY");
        // The span containing Y must be green and not bold.
        let y_span = html
            .split("<span")
            .find(|s| s.contains(">Y</span>"))
            .unwrap_or("");
        assert!(y_span.contains("color: #55FF55"), "got: {html}");
        assert!(!y_span.contains("font-weight: bold;"), "got: {html}");
    }
}
