use std::{io, net::IpAddr, sync::Arc};

use chrono::Local;
use colored::Colorize;
use database::{DatabaseWrapper, ServerInsert};
use diesel::{insert_into, RunQueryDsl};
use mc_lookup::{check_server, generate_random_ip};
use server_actions::{
    with_connection::get_extra_data,
    without_connection::{get_status, Status},
};
use tokio::sync::Mutex;

mod conn_wrapper;
mod database;
mod packets;
mod schema;
mod server_actions;

const MAX_WORKERS: usize = 512;

fn log_server_status(status: &Status, ip: &IpAddr) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();

    println!(
        "[{}] {} {} | {} {} | {} {}/{}",
        timestamp,
        "🌐 Address:".blue(),
        ip,
        "🛠  Version:".yellow(),
        status.version.name,
        "👥 Players:".green(),
        status.players.online,
        status.players.max,
    );
}

pub async fn handle_valid_ip(
    ip: &IpAddr,
    port: u16,
    db: Arc<Mutex<DatabaseWrapper>>,
) -> io::Result<()> {
    let status = get_status(format!("{}", ip), port).await?;
    log_server_status(&status, ip);

    let extra_data =
        get_extra_data(format!("{}", ip), port, status.version.protocol as i32).await?;

    let server_insert = ServerInsert {
        addr: format!("{}", ip),
        online: status.players.online as i32,
        max: status.players.max as i32,
        version_name: status.version.name,
        protocol: status.version.protocol as i32,
        license: extra_data.license,
        white_list: extra_data.white_list,
    };

    insert_into(schema::server::dsl::server)
        .values(server_insert)
        .execute(&mut db.lock().await.conn)
        .unwrap();

    // TODO: Add players

    Ok(())
}

async fn worker(db: Arc<Mutex<DatabaseWrapper>>) {
    loop {
        let addr = generate_random_ip();

        if check_server(&IpAddr::V4(addr), 25565).await {
            if let Err(_) = handle_valid_ip(&IpAddr::V4(addr), 25565, db.clone()).await {
                // println!("Err: {}", addr);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let db = DatabaseWrapper::establish();

    let mut workers = vec![];

    for _ in 0..MAX_WORKERS {
        workers.push(tokio::spawn(worker(db.clone())));
    }

    for task in workers {
        let _ = task.await;
    }
}
