use std::io;

use async_trait::async_trait;
use minecraft_protocol::{
    packet_builder::PacketBuilder, packet_reader::PacketReader, types::var_int::VarInt,
    UncompressedPacket,
};

pub trait PacketActions {
    fn serialize(&self) -> UncompressedPacket;
    fn deserialize(packet: UncompressedPacket) -> io::Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct Handshake {
    pub protocol: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}

impl PacketActions for Handshake {
    fn serialize(&self) -> UncompressedPacket {
        PacketBuilder::new(VarInt(0x00))
            .write(self.protocol.clone())
            .write(self.server_address.clone())
            .write(self.server_port)
            .write(self.next_state.clone())
            .build()
    }

    fn deserialize(packet: UncompressedPacket) -> io::Result<Self> {
        let mut reader = PacketReader::new(&packet);

        let protocol = reader.read()?;
        let server_address = reader.read()?;
        let server_port = reader.read()?;
        let next_state = reader.read()?;

        Ok(Handshake {
            protocol,
            server_address,
            server_port,
            next_state,
        })
    }
}

pub struct StatusRequest {}

impl PacketActions for StatusRequest {
    fn serialize(&self) -> UncompressedPacket {
        PacketBuilder::new(VarInt(0x00)).build()
    }

    fn deserialize(packet: UncompressedPacket) -> io::Result<Self> {
        Ok(StatusRequest {})
    }
}

pub struct StatusResponse {
    pub response: String,
}

#[async_trait]
impl PacketActions for StatusResponse {
    fn serialize(&self) -> UncompressedPacket {
        PacketBuilder::new(VarInt(0x00))
            .write(self.response.clone())
            .build()
    }

    fn deserialize(packet: UncompressedPacket) -> io::Result<Self> {
        let mut reader = PacketReader::new(&packet);

        let response = reader.read()?;

        Ok(StatusResponse { response })
    }
}

#[derive(Debug)]
pub struct LoginStart {
    pub name: String,
    pub uuid: u128,
}

impl PacketActions for LoginStart {
    fn serialize(&self) -> UncompressedPacket {
        PacketBuilder::new(VarInt(0x00))
            .write(self.name.clone())
            .write(self.uuid)
            .build()
    }

    fn deserialize(packet: UncompressedPacket) -> io::Result<Self> {
        let mut reader = PacketReader::new(&packet);

        let name = reader.read()?;
        let uuid = reader.read()?;

        Ok(LoginStart { name, uuid })
    }
}

pub struct SetCompression {
    pub threshold: VarInt,
}

impl PacketActions for SetCompression {
    fn serialize(&self) -> UncompressedPacket {
        todo!()
    }

    fn deserialize(packet: UncompressedPacket) -> io::Result<Self> {
        let mut reader = PacketReader::new(&packet);

        let threshold = reader.read()?;

        Ok(SetCompression { threshold })
    }
}