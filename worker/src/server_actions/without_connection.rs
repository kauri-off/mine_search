use std::time::{SystemTime, UNIX_EPOCH};

use mc_protocol::{
    packet::{RawPacket, UncompressedPacket},
    varint::VarInt,
};
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
