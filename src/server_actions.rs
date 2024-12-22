use std::io::{self, Error, ErrorKind};

use minecraft_protocol::{
    packet_builder::PacketBuilder, packet_reader::PacketReader, types::var_int::VarInt, Packet,
    UncompressedPacket,
};
use serde::Deserialize;
use tokio::net::TcpStream;

use crate::{
    mc_session::ConnectionWrapper,
    packets::{
        Handshake, LoginStart, PacketActions, SetCompression, StatusRequest, StatusResponse,
    },
};

#[derive(Deserialize, Debug)]
pub struct Status {
    pub players: Players,
    pub version: Version,
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

pub async fn ping_server(server_addr: &str, handshake_addr: &str, port: u16) -> io::Result<Status> {
    let connection = TcpStream::connect(server_addr).await?;
    let mut wrapper = ConnectionWrapper { conn: connection };

    wrapper
        .write_packet(Handshake {
            protocol: VarInt(765),
            server_address: handshake_addr.to_string(),
            server_port: port,
            next_state: VarInt(1),
        })
        .await?;

    wrapper.write_packet(StatusRequest {}).await?;

    let response: StatusResponse = wrapper.read_packet().await?;

    let value: Status =
        serde_json::from_str(&response.response).map_err(|_| ErrorKind::InvalidData)?;
    Ok(value)
}

#[derive(Debug)]
pub struct ExtraData {
    pub license: bool,
}
pub async fn get_extra_data(
    server_addr: &str,
    handshake_addr: &str,
    port: u16,
    protocol: i32,
) -> io::Result<ExtraData> {
    let connection = TcpStream::connect(server_addr).await?;
    let mut wrapper = ConnectionWrapper { conn: connection };

    wrapper
        .write_packet(Handshake {
            protocol: VarInt(protocol),
            server_address: handshake_addr.to_string(),
            server_port: port,
            next_state: VarInt(2),
        })
        .await?;

    get_login_start(protocol).write(&mut wrapper.conn).await?;

    let mut threshold = None;
    let packet = Packet::read_uncompressed(&mut wrapper.conn).await?;

    let packet = if packet.packet_id.0 == 0x03 {
        threshold = Some(SetCompression::deserialize(packet)?.threshold.0);
        Packet::read(&mut wrapper.conn, threshold).await?
    } else {
        Packet::UnCompressed(packet)
    };

    if packet.packet_id().await?.0 == 0x00 {
        let reason: String = if let Packet::UnCompressed(p) = packet {
            PacketReader::new(&p).read()?
        } else {
            "error".to_string()
        };

        return Err(Error::new(ErrorKind::InvalidData, reason));
    }

    if packet.packet_id().await?.0 != 0x02 {
        return Ok(ExtraData { license: true });
    }

    // let mut auth = false;

    // let start_time = time::Instant::now();
    // loop {
    //     if start_time.elapsed() > Duration::from_secs(2) {
    //         break;
    //     }
    //     let packet = Packet::read(&mut wrapper.conn, threshold).await?;
    //     println!("{:?}", packet);
    // }

    Ok(ExtraData { license: false })
}

fn get_login_start(protocol: i32) -> UncompressedPacket {
    if protocol > 763 {
        LoginStart {
            name: "LookupPlayer".to_string(),
            uuid: 0x1f6969963dace4643bfa0c99a4db549,
        }
        .serialize()
    } else if protocol > 761 {
        PacketBuilder::new(VarInt(0x00))
            .write("LookupPlayer".to_string())
            .write_option(Some(0x1f6969963dace4643bfa0c99a4db549 as u128))
            .build()
    } else if protocol > 758 {
        PacketBuilder::new(VarInt(0x00))
            .write("LookupPlayer".to_string())
            .write_option::<bool>(None)
            .write_option(Some(0x1f6969963dace4643bfa0c99a4db549 as u128))
            .build()
    } else {
        PacketBuilder::new(VarInt(0x00))
            .write("LookupPlayer".to_string())
            .build()
    }
}
