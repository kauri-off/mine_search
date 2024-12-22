use std::{io, net::IpAddr, sync::Arc};

use mc_lookup::{check_server, generate_random_ip};
use server_actions::{get_extra_data, ping_server};

mod mc_session;
mod packets;
mod server_actions;

const MAX_WORKERS: usize = 512;

pub struct DatabaseWrapper {}

pub async fn handle_valid_ip(ip: &IpAddr, port: u16, _db: Arc<DatabaseWrapper>) -> io::Result<()> {
    let status = ping_server(&format!("{}:{}", ip, port), &format!("{}", ip), port).await?;

    let extra_data = get_extra_data(
        &format!("{}:{}", ip, port),
        &format!("{}", ip),
        port,
        status.version.protocol as i32,
    )
    .await?;

    println!(
        "{} - {}/{} : {} : {:?}",
        ip, status.players.online, status.players.max, extra_data.license, status.players
    );

    Ok(())
}

async fn worker(db: Arc<DatabaseWrapper>) {
    loop {
        let addr = generate_random_ip();

        if check_server(&IpAddr::V4(addr), 25565).await {
            let _ = handle_valid_ip(&IpAddr::V4(addr), 25565, db.clone()).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let db = Arc::new(DatabaseWrapper {});

    // handle_valid_ip(&"127.0.0.1".parse().unwrap(), 25565, db.clone())
    //     .await
    //     .unwrap();

    let mut workers = vec![];

    for _ in 0..MAX_WORKERS {
        workers.push(tokio::spawn(worker(db.clone())));
    }

    for task in workers {
        let _ = task.await;
    }
}
