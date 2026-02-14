use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{net::TcpStream, time::timeout};

pub async fn check_server(ip: &IpAddr, port: u16) -> bool {
    let addr = format!("{}:{}", ip, port);

    match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
        Ok(t) => t.is_ok(),
        Err(_) => false,
    }
}

pub fn generate_random_ip() -> Ipv4Addr {
    let mut rng = rand::rng();

    loop {
        // Генерируем случайный адрес
        let first_byte: u8 = rng.random_range(1..=223); // от 1 до 223, чтобы не попасть в частные диапазоны
        let second_byte: u8 = rng.random_range(0..=255);
        let third_byte: u8 = rng.random_range(0..=255);
        let fourth_byte: u8 = rng.random_range(0..=255);

        let ip = Ipv4Addr::new(first_byte, second_byte, third_byte, fourth_byte);

        // Проверяем, что IP не попадает в частные диапазоны
        if !is_private_ip(&ip) {
            return ip;
        }
    }
}

fn is_private_ip(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();

    // Проверка для частных диапазонов:
    // 10.0.0.0 - 10.255.255.255
    if octets[0] == 10 {
        return true;
    }

    // Проверка для частных диапазонов:
    // 127.0.0.0 - 127.255.255.255
    if octets[0] == 127 {
        return true;
    }

    // 172.16.0.0 - 172.31.255.255
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return true;
    }
    // 192.168.0.0 - 192.168.255.255
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }

    false
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
