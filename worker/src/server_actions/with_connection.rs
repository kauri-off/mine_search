use minecraft_protocol::{
    packet::{RawPacket, UncompressedPacket},
    varint::VarInt,
};
use serde_json::Value;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::packets::*;

#[derive(Debug)]
pub struct ExtraData {
    pub license: bool,
    pub disconnect_reason: Option<Value>,
}

pub async fn get_extra_data(ip: String, port: u16, protocol: i32) -> anyhow::Result<ExtraData> {
    let mut conn = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    let handshake = c2s::Handshake {
        protocol_version: VarInt(protocol),
        server_address: ip,
        server_port: port,
        intent: VarInt(2),
    };

    UncompressedPacket::from_packet(&handshake)?
        .write_async(&mut conn)
        .await?;

    let login_start = c2s::LoginStart {
        name: "Notch".to_string(),
        uuid: Uuid::from_u128(0x069a79f444e94726a5befca90e38aaf5),
    };

    login_start
        .raw_by_protocol(protocol)?
        .write_async(&mut conn)
        .await?;

    let mut threshold = None;

    loop {
        let packet = RawPacket::read_async(&mut conn)
            .await?
            .uncompress(threshold)?;

        match packet.packet_id {
            s2c::LoginDisconnect::PACKET_ID => {
                let reason: String = packet.deserialize_payload::<s2c::LoginDisconnect>()?.reason;
                return Ok(ExtraData {
                    license: false,
                    disconnect_reason: Some(serde_json::from_str::<Value>(&reason)?),
                });
            }
            s2c::EncryptionRequest::PACKET_ID => {
                return Ok(ExtraData {
                    license: true,
                    disconnect_reason: None,
                });
            }
            s2c::LoginFinished::PACKET_ID => {
                return Ok(ExtraData {
                    license: false,
                    disconnect_reason: None,
                });
            }
            s2c::SetCompression::PACKET_ID => {
                threshold = Some(
                    packet
                        .deserialize_payload::<s2c::SetCompression>()?
                        .threshold
                        .0,
                );
            }
            _ => {
                return Err(anyhow::anyhow!("error"));
            }
        }
    }
}
