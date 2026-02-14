use minecraft_protocol::{packet::RawPacket, varint::VarInt};
use tokio::net::TcpStream;

use crate::packets::*;

#[derive(Debug)]
pub struct ExtraData {
    pub license: bool,
    pub white_list: Option<bool>,
}

pub async fn get_extra_data(ip: String, port: u16, protocol: i32) -> anyhow::Result<ExtraData> {
    let mut conn = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    c2s::Handshake {
        protocol_version: VarInt(protocol),
        server_address: ip,
        server_port: port,
        intent: VarInt(2),
    }
    .as_uncompressed()?
    .to_raw_packet()?
    .write(&mut conn)
    .await?;

    c2s::LoginStart {
        name: "Notch".to_string(),
        uuid: 0x069a79f444e94726a5befca90e38aaf5,
    }
    .raw_by_protocol(protocol)
    .write(&mut conn)
    .await?;

    let mut threshold = None;

    loop {
        let packet = RawPacket::read(&mut conn)
            .await?
            .try_uncompress(threshold)?;

        match packet {
            Some(t) if t.packet_id.0 == 0 => {
                let reason: String = t.convert::<s2c::LoginDisconnect>()?.reason;
                if reason == "{\"text\":\"You are not whitelisted on this server!\"}" {
                    return Ok(ExtraData {
                        license: false,
                        white_list: Some(true),
                    });
                } else {
                    return Ok(ExtraData {
                        license: false,
                        white_list: None,
                    });
                }
            }
            Some(t) if t.packet_id.0 == 1 => {
                return Ok(ExtraData {
                    license: true,
                    white_list: None,
                });
            }
            Some(t) if t.packet_id.0 == 2 => {
                return Ok(ExtraData {
                    license: false,
                    white_list: Some(false),
                });
            }
            Some(t) if t.packet_id.0 == 3 => {
                threshold = Some(t.convert::<s2c::SetCompression>()?.threshold.0);
            }
            _ => {
                return Err(anyhow::anyhow!("error"));
            }
        }
    }
}
