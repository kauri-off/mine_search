use std::time::{SystemTime, UNIX_EPOCH};

use mc_protocol::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;

use crate::packets::*;

#[derive(Deserialize, Debug)]
pub struct Status {
    pub players: Players,
    pub version: Version,
    pub description: Value,
    #[serde(rename = "forgeData")]
    pub forge_data: Option<Value>,
    pub modinfo: Option<Value>,
    pub favicon: Option<String>,
}

/// Loader tokens in `version.name` implying a client needs a loader/mods to join.
/// "forge" also matches "neoforge"; hybrids (mohist/arclight/magma/catserver/banner)
/// carry their own name and are intentionally absent — they admit vanilla clients.
const LOADER_KEYWORDS: [&str; 5] = ["forge", "neoforge", "fabric", "quilt", "fml"];

impl Status {
    /// True when a vanilla client cannot join without downloading mods.
    /// `(heuristic || markers) && !server_side_only` — a loader named in the version
    /// string OR an advertised Forge/FML mod list flags the server, unless the Forge
    /// channels prove the mods are all optional (server-side-only), which cancels it.
    pub fn requires_mods(&self) -> bool {
        let name = self.version.name.to_ascii_lowercase();
        let heuristic = LOADER_KEYWORDS.iter().any(|kw| name.contains(kw));

        let markers = self.forge_data.is_some() || self.has_legacy_mods();

        (heuristic || markers) && !self.is_server_side_only()
    }

    /// True when legacy `modinfo` advertises at least one mod. An empty `modList`
    /// (commonly spoofed as `{ "type": "FML", "modList": [] }` to look
    /// Forge-friendly) requires nothing of the client, so a vanilla player can
    /// still join — that must not flag the server.
    fn has_legacy_mods(&self) -> bool {
        self.modinfo
            .as_ref()
            .and_then(|m| m.get("modList"))
            .and_then(Value::as_array)
            .is_some_and(|list| !list.is_empty())
    }

    /// True only when `forgeData` lists channels and *every* one is `required: false`.
    /// Truncated FML3 pings (channels live in the binary `"d"` blob → array empty/absent)
    /// and legacy `modinfo` (no channel data) fall through to `false`.
    fn is_server_side_only(&self) -> bool {
        let Some(channels) = self
            .forge_data
            .as_ref()
            .and_then(|f| f.get("channels"))
            .and_then(Value::as_array)
        else {
            return false;
        };
        !channels.is_empty()
            && channels
                .iter()
                .all(|c| c.get("required").and_then(Value::as_bool) == Some(false))
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct Players {
    pub online: i64,
    pub max: i64,
    pub sample: Option<Vec<Player>>,
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i64,
}

pub async fn get_status(
    ip: &str,
    port: u16,
    tcp_stream: Option<TcpStream>,
) -> anyhow::Result<(Status, Option<i64>)> {
    let mut tcp_stream = match tcp_stream {
        Some(t) => t,
        None => TcpStream::connect(&format!("{}:{}", ip, port)).await?,
    };

    let handshake = c2s::Handshake {
        protocol_version: VarInt(765),
        server_address: ip.to_string(),
        server_port: port,
        intent: VarInt(1),
    };

    UncompressedPacket::from_packet(&handshake)?
        .write_async(&mut tcp_stream)
        .await?;

    let status_request = c2s::StatusRequest {};

    UncompressedPacket::from_packet(&status_request)?
        .write_async(&mut tcp_stream)
        .await?;

    let response: s2c::StatusResponse = RawPacket::read_async(&mut tcp_stream)
        .await?
        .as_uncompressed()?
        .deserialize_payload()?;

    let value: Status = serde_json::from_str(&response.response)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH")
        .as_millis() as i64;

    let ping_request = c2s::PingRequest { timestamp };

    let _ = UncompressedPacket::from_packet(&ping_request)?
        .write_async(&mut tcp_stream)
        .await;

    if RawPacket::read_async(&mut tcp_stream).await.is_ok() {
        let ping_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_millis() as i64
            - timestamp;

        Ok((value, Some(ping_ms)))
    } else {
        Ok((value, None))
    }
}

#[cfg(test)]
mod tests {
    use super::Status;
    use serde_json::json;

    /// Builds a `Status` from a `version.name` and an optional raw status body
    /// (merged in), so tests can focus on the fields the detector reads.
    fn status(version_name: &str, extra: serde_json::Value) -> Status {
        let mut body = json!({
            "players": { "online": 0, "max": 20 },
            "version": { "name": version_name, "protocol": 765 },
            "description": "",
        });
        body.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_value(body).expect("valid Status")
    }

    #[test]
    fn vanilla_plain_name_no_markers() {
        assert!(!status("1.20.1", json!({})).requires_mods());
    }

    #[test]
    fn loader_named_in_version_no_markers() {
        assert!(status("NeoForge 21.1", json!({})).requires_mods());
    }

    #[test]
    fn loader_named_but_server_side_only_is_cancelled() {
        let s = status(
            "Forge 1.20.1",
            json!({ "forgeData": { "channels": [
                { "res": "x:main", "version": "1", "required": false }
            ] } }),
        );
        assert!(!s.requires_mods());
    }

    #[test]
    fn plain_name_with_required_channel_needs_mods() {
        let s = status(
            "1.20.1",
            json!({ "forgeData": { "channels": [
                { "res": "x:main", "version": "1", "required": true }
            ] } }),
        );
        assert!(s.requires_mods());
    }

    #[test]
    fn plain_name_with_legacy_modinfo_needs_mods() {
        let s = status(
            "1.12.2",
            json!({ "modinfo": { "type": "FML", "modList": [
                { "modid": "forge", "version": "14.23" }
            ] } }),
        );
        assert!(s.requires_mods());
    }

    #[test]
    fn plain_name_with_truncated_fml3_needs_mods() {
        let s = status(
            "1.20.1",
            json!({ "forgeData": { "channels": [], "truncated": true, "d": "abc" } }),
        );
        assert!(s.requires_mods());
    }

    #[test]
    fn plain_name_with_empty_modinfo_modlist_is_not_modded() {
        let s = status(
            "1.21.1",
            json!({ "modinfo": { "type": "FML", "modList": [] } }),
        );
        assert!(!s.requires_mods());
    }

    /// Live probe — pings a real server and dumps the fields the detector reads,
    /// so we can see *why* it gets flagged. Ignored by default (needs network).
    /// Run with: `cargo test -p worker probe_real_server -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn probe_real_server() {
        let ip = "";
        let port = 25565;

        let (status, ping) = super::get_status(ip, port, None)
            .await
            .expect("status ping failed");

        eprintln!("=== {ip}:{port} (ping {ping:?}ms) ===");
        eprintln!("version.name     = {:?}", status.version.name);
        eprintln!("version.protocol = {}", status.version.protocol);
        eprintln!("forge_data       = {}", status.forge_data.is_some());
        eprintln!("modinfo          = {}", status.modinfo.is_some());
        if let Some(fd) = &status.forge_data {
            eprintln!(
                "forgeData JSON   = {}",
                serde_json::to_string_pretty(fd).unwrap()
            );
        }
        if let Some(mi) = &status.modinfo {
            eprintln!(
                "modinfo JSON     = {}",
                serde_json::to_string_pretty(mi).unwrap()
            );
        }
        eprintln!("is_server_side_only = {}", status.is_server_side_only());
        eprintln!(">>> requires_mods   = {}", status.requires_mods());
    }
}
