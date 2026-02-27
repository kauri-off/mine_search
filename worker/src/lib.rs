use std::net::Ipv4Addr;

use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn generate_random_ip(rng: &mut ChaCha8Rng) -> Ipv4Addr {
    loop {
        let raw_ip: u32 = rng.random();
        let ip = Ipv4Addr::from(raw_ip);
        let octets = ip.octets();

        if octets[0] == 0 || octets[0] > 223 {
            continue;
        }

        if !is_reserved_ip(octets) {
            return ip;
        }
    }
}

#[inline(always)]
fn is_reserved_ip(octets: [u8; 4]) -> bool {
    match octets[0] {
        10 => true,                                                    // 10.0.0.0/8
        127 => true,                                                   // 127.0.0.0/8 (loopback)
        172 => octets[1] >= 16 && octets[1] <= 31,                     // 172.16.0.0/12
        192 => octets[1] == 168 || (octets[1] == 0 && octets[2] == 2), // 192.168.0.0/16 + 192.0.2.0/24
        169 => octets[1] == 254, // 169.254.0.0/16 (link-local)
        100 => octets[1] >= 64 && octets[1] <= 127, // 100.64.0.0/10 (CGNAT)
        198 => {
            (octets[1] == 18 || octets[1] == 19)            // 198.18.0.0/15 (benchmarking)
            || (octets[1] == 51 && octets[2] == 100) // 198.51.100.0/24 (documentation)
        }
        203 => octets[1] == 0 && octets[2] == 113, // 203.0.113.0/24 (documentation)
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
