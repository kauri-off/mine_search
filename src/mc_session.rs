use std::io;

use minecraft_protocol::Packet;
use tokio::net::TcpStream;

use crate::packets::PacketActions;

pub struct ConnectionWrapper {
    pub conn: TcpStream,
}

impl ConnectionWrapper {
    pub async fn read_packet<T: PacketActions>(&mut self) -> io::Result<T> {
        let packet = Packet::read_uncompressed(&mut self.conn).await?;
        T::deserialize(packet)
    }

    pub async fn write_packet<T: PacketActions>(&mut self, packet: T) -> io::Result<()> {
        packet.serialize().write(&mut self.conn).await
    }

    pub async fn write<T: PacketActions>(
        &mut self,
        packet: T,
        threshold: Option<i32>,
    ) -> io::Result<()> {
        Packet::UnCompressed(packet.serialize())
            .write(&mut self.conn, threshold)
            .await
    }
}
