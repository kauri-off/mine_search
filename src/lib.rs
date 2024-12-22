use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use rand::Rng;
use tokio::{net::TcpStream, time::timeout};

pub async fn check_server(ip: &IpAddr, port: u16) -> bool {
    let addr = format!("{}:{}", ip, port);

    match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
        Ok(t) => t.is_ok(),
        Err(_) => false,
    }
}

pub fn generate_random_ip() -> Ipv4Addr {
    let mut rng = rand::thread_rng();

    loop {
        // Генерируем случайный адрес
        let first_byte: u8 = rng.gen_range(1..=223); // от 1 до 223, чтобы не попасть в частные диапазоны
        let second_byte: u8 = rng.gen_range(0..=255);
        let third_byte: u8 = rng.gen_range(0..=255);
        let fourth_byte: u8 = rng.gen_range(0..=255);

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
