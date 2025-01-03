use std::io::{self, Error, ErrorKind};

use minecraft_protocol::{packet_reader::PacketReader, types::var_int::VarInt, Packet};
use tokio::net::TcpStream;

use crate::{
    conn_wrapper::ConnectionWrapper,
    packets::{Handshake, LoginStart, PacketActions, SetCompression},
};

#[derive(Debug)]
pub struct ExtraData {
    pub license: bool,
    pub white_list: Option<bool>,
}

pub async fn get_extra_data(ip: String, port: u16, protocol: i32) -> io::Result<ExtraData> {
    let mut conn = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    conn.write_packet(Handshake {
        protocol: VarInt(protocol),
        server_address: ip,
        server_port: port,
        next_state: VarInt(2),
    })
    .await?;

    LoginStart {
        name: "LookupPlayer".to_string(),
        uuid: 0x1f6969963dace4643bfa0c99a4db549,
    }
    .get_by_protocol(protocol)
    .write(&mut conn)
    .await
    .unwrap();

    // let mut threshold = None;
    let packet = Packet::read_uncompressed(&mut conn).await?;

    let packet = if packet.packet_id.0 == 0x03 {
        let threshold = Some(SetCompression::deserialize(packet)?.threshold.0);
        Packet::read(&mut conn, threshold).await?
    } else {
        Packet::UnCompressed(packet)
    };

    if packet.packet_id().await?.0 == 0x00 {
        let reason: String = if let Packet::UnCompressed(p) = packet {
            PacketReader::new(&p).read()?
        } else {
            "error".to_string()
        };
        if reason == "{\"text\":\"You are not whitelisted on this server!\"}" {
            return Ok(ExtraData {
                license: false,
                white_list: Some(true),
            });
        }

        return Err(Error::new(ErrorKind::InvalidData, reason));
    }

    if packet.packet_id().await?.0 != 0x02 {
        return Ok(ExtraData {
            license: true,
            white_list: None,
        });
    }

    Ok(ExtraData {
        license: false,
        white_list: Some(false),
    })
}
