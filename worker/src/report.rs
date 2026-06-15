//! Transport-neutral scan result. Both sinks (diesel direct-write and gRPC
//! streaming) consume this, so the scanning logic doesn't know how results are
//! persisted.

use std::time::Duration;

use serde_json::Value;
use tokio::{net::TcpStream, time::timeout};

use crate::server_actions::{with_connection::get_extra_data, without_connection::get_status};

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub ip: String,
    pub port: i32,
    pub version_name: String,
    pub protocol: i32,
    pub description: Value,
    pub players_online: i32,
    pub players_max: i32,
    pub player_names: Vec<String>,
    pub requires_mods: bool,
    pub favicon: Option<String>,
    pub ping: Option<i64>,
    pub extra: Option<ScanExtra>,
}

#[derive(Debug, Clone)]
pub struct ScanExtra {
    pub is_online_mode: bool,
    pub disconnect_reason: Option<Value>,
}

/// Probes a server's status (and optionally its login handshake) and assembles a
/// [`ScanReport`].
///
/// - `fetch_extra` controls whether the login handshake (online-mode detection)
///   is attempted.
/// - `require_extra` makes a failed handshake fatal (discovery path, which
///   mirrors the old `handle_valid_ip`); otherwise handshake failures are
///   ignored (update path).
pub async fn probe(
    ip: &str,
    port: u16,
    tcp_stream: Option<TcpStream>,
    fetch_extra: bool,
    require_extra: bool,
) -> anyhow::Result<ScanReport> {
    let (status, ping) = get_status(ip, port, tcp_stream).await?;
    let requires_mods = status.requires_mods();

    let extra = if fetch_extra {
        match get_extra_data(ip.to_string(), port, status.version.protocol as i32).await {
            Ok(e) => Some(ScanExtra {
                is_online_mode: e.is_online_mode,
                disconnect_reason: e.disconnect_reason,
            }),
            Err(err) => {
                if require_extra {
                    return Err(err);
                }
                None
            }
        }
    } else {
        None
    };

    let player_names = status
        .players
        .sample
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.name)
        .collect();

    Ok(ScanReport {
        ip: ip.to_string(),
        port: port as i32,
        version_name: status.version.name,
        protocol: status.version.protocol as i32,
        description: status.description,
        players_online: status.players.online as i32,
        players_max: status.players.max as i32,
        player_names,
        requires_mods,
        favicon: status.favicon,
        ping,
        extra,
    })
}

/// Opens a TCP connection to a candidate address, with a short timeout.
pub async fn check_server(ip: &str, port: u16) -> anyhow::Result<TcpStream> {
    let addr = format!("{}:{}", ip, port);
    Ok(timeout(Duration::from_millis(750), TcpStream::connect(&addr)).await??)
}
