use minecraft_protocol::{packet::RawPacket, varint::VarInt};
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
    #[serde(rename = "isModded")]
    pub is_modded: Option<bool>,
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
) -> anyhow::Result<Status> {
    let mut tcp_stream = match tcp_stream {
        Some(t) => t,
        None => TcpStream::connect(&format!("{}:{}", ip, port)).await?,
    };

    c2s::Handshake {
        protocol_version: VarInt(765),
        server_address: ip.to_string(),
        server_port: port,
        intent: VarInt(1),
    }
    .as_uncompressed()?
    .to_raw_packet()?
    .write(&mut tcp_stream)
    .await?;

    c2s::StatusRequest {}
        .as_uncompressed()?
        .to_raw_packet()?
        .write(&mut tcp_stream)
        .await?;

    let response: s2c::StatusResponse = RawPacket::read(&mut tcp_stream)
        .await?
        .as_uncompressed()?
        .convert()?;

    let value: Status = serde_json::from_str(&response.response)?;
    Ok(value)
}
