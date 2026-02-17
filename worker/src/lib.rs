use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{net::TcpStream, time::timeout};

pub async fn check_server(ip: &IpAddr, port: u16) -> anyhow::Result<TcpStream> {
    let addr = format!("{}:{}", ip, port);

    Ok(timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await??)
}

pub fn generate_random_ip(rng: &mut ChaCha8Rng) -> Ipv4Addr {
    loop {
        let raw_ip: u32 = rng.random();
        let ip = Ipv4Addr::from(raw_ip);
        let octets = ip.octets();

        if octets[0] == 0 || octets[0] > 223 {
            continue;
        }

        if !is_private_ip(octets) {
            return ip;
        }
    }
}

#[inline(always)]
fn is_private_ip(octets: [u8; 4]) -> bool {
    match octets[0] {
        10 => true,                                // 10.0.0.0/8
        127 => true,                               // 127.0.0.0/8 (Loopback)
        172 => octets[1] >= 16 && octets[1] <= 31, // 172.16.0.0/12
        192 => octets[1] == 168,                   // 192.168.0.0/16
        169 => octets[1] == 254,                   // 169.254.0.0/16 (Link-local)
        _ => false,
    }
}

pub fn description_to_str(description: Value) -> Result<String, serde_json::Error> {
    let chat_object: ChatObject = serde_json::from_value(description)?;
    Ok(chat_object.get_motd())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    Object(ChatComponentObject),
    Array(Vec<ChatObject>),
    JsonPrimitive(Value),
}

impl ChatObject {
    pub fn get_motd(&self) -> String {
        match self {
            ChatObject::Object(chat_component_object) => {
                let mut result = String::new();

                if let Some(text) = &chat_component_object.text {
                    result += text.as_str();
                }

                if let Some(extra) = &chat_component_object.extra {
                    for object in extra {
                        result += &object.get_motd();
                    }
                }

                result
            }
            ChatObject::Array(vec) => {
                let mut result = String::new();

                for object in vec {
                    result += &object.get_motd();
                }

                result
            }
            ChatObject::JsonPrimitive(value) => value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatComponentObject {
    pub text: Option<String>,
    pub extra: Option<Vec<ChatObject>>,
}
