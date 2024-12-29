use std::io::{self, ErrorKind};

use minecraft_protocol::types::var_int::VarInt;
use serde::Deserialize;
use tokio::net::TcpStream;

use crate::{
    conn_wrapper::ConnectionWrapper,
    packets::{Handshake, StatusRequest, StatusResponse},
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

pub async fn get_status(ip: String, port: u16) -> io::Result<Status> {
    let mut conn = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    conn.write_packet(Handshake {
        protocol: VarInt(765),
        server_address: ip,
        server_port: port,
        next_state: VarInt(1),
    })
    .await?;

    conn.write_packet(StatusRequest {}).await?;

    let response: StatusResponse = conn.read_packet().await?;

    let value: Status =
        serde_json::from_str(&response.response).map_err(|_| ErrorKind::InvalidData)?;
    Ok(value)
}
