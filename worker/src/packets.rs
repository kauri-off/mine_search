use minecraft_protocol::{Packet, varint::VarInt};

#[allow(unused)]
pub mod c2s {
    use minecraft_protocol::packet::RawPacket;

    use super::*;
    // ----------- HANDSHAKING -----------
    #[derive(Packet, Debug)]
    #[packet(0x00)]
    pub struct Handshake {
        pub protocol_version: VarInt,
        pub server_address: String,
        pub server_port: u16,
        pub intent: VarInt,
    }

    // ----------- STATUS -----------

    #[derive(Packet)]
    #[packet(0x00)]
    pub struct StatusRequest {}

    // ----------- LOGIN -----------
    #[derive(Packet, Debug)]
    #[packet(0x00)]
    pub struct LoginStart {
        pub name: String,
        pub uuid: u128,
    }

    impl LoginStart {
        pub fn raw_by_protocol(&self, protocol: i32) -> RawPacket {
            if protocol > 763 {
                self.as_uncompressed().unwrap().to_raw_packet().unwrap()
            } else if protocol > 761 {
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload);
                minecraft_protocol::ser::Serialize::serialize(&true, &mut payload);
                minecraft_protocol::ser::Serialize::serialize(&self.uuid, &mut payload);

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
                .unwrap()
            } else if protocol > 758 {
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload);
                minecraft_protocol::ser::Serialize::serialize(&false, &mut payload);
                minecraft_protocol::ser::Serialize::serialize(&true, &mut payload);
                minecraft_protocol::ser::Serialize::serialize(&self.uuid, &mut payload);

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
                .unwrap()
            } else {
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload);

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
                .unwrap()
            }
        }
    }

    #[derive(Packet, Debug)]
    #[packet(0x14)]
    pub struct Look {
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug)]
    #[packet(0x13)]
    pub struct PositionLook {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug)]
    #[packet(0x12)]
    pub struct Position {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug, Clone)]
    #[packet(0x07)]
    pub struct Transaction {
        pub window_id: i8,
        pub action: i16,
        pub accepted: bool,
    }
}

pub mod s2c {
    use super::*;

    // ----------- STATUS -----------
    #[derive(Packet)]
    #[packet(0x00)]
    pub struct StatusResponse {
        pub response: String,
    }

    // ----------- LOGIN -----------
    #[derive(Packet, Debug)]
    #[packet(0x00)]
    pub struct LoginDisconnect {
        pub reason: String,
    }

    #[derive(Packet, Debug)]
    #[packet(0x03)]
    pub struct SetCompression {
        pub threshold: VarInt,
    }
}
