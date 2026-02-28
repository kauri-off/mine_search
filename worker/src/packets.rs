use minecraft_protocol::{Packet, varint::VarInt};

pub mod c2s {
    use minecraft_protocol::packet::{PacketError, RawPacket, UncompressedPacket};
    use uuid::Uuid;

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
        pub uuid: Uuid,
    }

    impl LoginStart {
        pub fn raw_by_protocol(&self, protocol: i32) -> Result<RawPacket, PacketError> {
            if protocol >= 764 {
                // 1.20.2+ : name + UUID (always present, no boolean prefix)
                UncompressedPacket::from_packet(self)
                    .unwrap()
                    .to_raw_packet()
            } else if protocol >= 761 {
                // 1.19.3 – 1.20.1 : name + has_uuid (bool) + UUID
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload)?;
                minecraft_protocol::ser::Serialize::serialize(&Some(self.uuid), &mut payload)?;

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
            } else if protocol == 760 {
                // 1.19.1 – 1.19.2 : name + has_sig_data (false) + has_uuid (true) + UUID
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload)?;
                minecraft_protocol::ser::Serialize::serialize(&false, &mut payload)?; // has_sig_data
                minecraft_protocol::ser::Serialize::serialize(&Some(self.uuid), &mut payload)?; // has_uuid + UUID

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
            } else if protocol == 759 {
                // 1.19 : name + has_uuid (false) — UUID omitted
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload)?;
                minecraft_protocol::ser::Serialize::serialize(&false, &mut payload)?; // has_uuid

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
            } else {
                // < 1.19 : name only
                let mut payload = Vec::new();
                minecraft_protocol::ser::Serialize::serialize(&self.name, &mut payload)?;

                minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload,
                }
                .to_raw_packet()
            }
        }
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
    #[packet(0x01)]
    pub struct EncryptionRequest {}

    #[derive(Packet, Debug)]
    #[packet(0x02)]
    pub struct LoginFinished {}

    #[derive(Packet, Debug)]
    #[packet(0x03)]
    pub struct SetCompression {
        pub threshold: VarInt,
    }
}
