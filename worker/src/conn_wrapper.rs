use std::io;

use minecraft_protocol::Packet;
use tokio::net::TcpStream;

use crate::packets::PacketActions;

pub trait ConnectionWrapper {
    async fn read_packet<T: PacketActions>(&mut self) -> io::Result<T>;
    async fn write_packet<T: PacketActions>(&mut self, packet: T) -> io::Result<()>;
    #[allow(dead_code)]
    async fn write<T: PacketActions>(
        &mut self,
        packet: T,
        threshold: Option<i32>,
    ) -> io::Result<()>;
}

impl ConnectionWrapper for TcpStream {
    async fn read_packet<T: PacketActions>(&mut self) -> io::Result<T> {
        let packet = Packet::read_uncompressed(self).await?;
        T::deserialize(packet)
    }

    async fn write_packet<T: PacketActions>(&mut self, packet: T) -> io::Result<()> {
        packet.serialize().write(self).await
    }

    async fn write<T: PacketActions>(
        &mut self,
        packet: T,
        threshold: Option<i32>,
    ) -> io::Result<()> {
        Packet::UnCompressed(packet.serialize())
            .write(self, threshold)
            .await
    }
}
