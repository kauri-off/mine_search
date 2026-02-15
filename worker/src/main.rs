use std::{collections::HashSet, env, net::IpAddr, sync::Arc, time::Duration};

use chrono::{Local, Utc};
use colored::Colorize;
use database::DatabaseWrapper;
use diesel::{dsl::insert_into, prelude::*};
use diesel_async::RunQueryDsl;
use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use server_actions::{with_connection::get_extra_data, without_connection::get_status};
use tokio::{sync::Semaphore, time::timeout};
use worker::{check_server, description_to_str, generate_random_ip};

use db_schema::{
    models::{
        data::{DataInsert, DataModelMini},
        servers::{ServerInsert, ServerModel, ServerModelMini, ServerUpdate},
    },
    schema,
};

mod database;
mod packets;
mod server_actions;

pub async fn handle_valid_ip(
    ip: &IpAddr,
    port: u16,
    db: Arc<DatabaseWrapper>,
) -> anyhow::Result<()> {
    let status = get_status(&format!("{}", ip), port).await?;

    let extra_data =
        get_extra_data(format!("{}", ip), port, status.version.protocol as i32).await?;

    let server_insert = ServerInsert {
        ip: &format!("{}", ip),
        port: port as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        license: extra_data.license,
        white_list: extra_data.white_list,
    };

    let mut conn = db.pool.get().await.unwrap();

    let server: ServerModel = insert_into(schema::servers::table)
        .values(server_insert)
        .on_conflict(schema::servers::ip)
        .do_nothing()
        .returning(ServerModel::as_returning())
        .get_result(&mut conn)
        .await?;

    let data_insert = DataInsert {
        server_id: server.id,
        online: status.players.online as i32,
        max: status.players.max as i32,
        players: &json!(
            status
                .players
                .sample
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.name)
                .collect::<Vec<String>>()
        ),
    };

    insert_into(schema::data::table)
        .values(data_insert)
        .execute(&mut conn)
        .await?;

    let timestamp = Local::now().format("%H:%M:%S").to_string();

    println!(
        "[{}] {} {} | {} {} | {} {}/{} | {} | {} {}",
        timestamp,
        "🌐",
        ip.to_string().blue(),
        "🛠 ",
        status.version.name.yellow(),
        "👥",
        status.players.online.to_string().green(),
        status.players.max,
        if extra_data.license {
            "yes".red()
        } else {
            "no".green()
        },
        "🚀",
        description_to_str(status.description).unwrap_or("".to_string())
    );
    Ok(())
}

async fn worker(db: Arc<DatabaseWrapper>) {
    let mut rng = ChaCha8Rng::try_from_rng(&mut SysRng).unwrap();

    loop {
        let ip = IpAddr::V4(generate_random_ip(&mut rng));

        if check_server(&ip, 25565).await {
            let _ = timeout(
                Duration::from_secs(5),
                handle_valid_ip(&ip, 25565, db.clone()),
            )
            .await;
        }
    }
}

async fn updater(db: Arc<DatabaseWrapper>) {
    loop {
        println!("Updating...");

        let servers: Vec<ServerModelMini> = schema::servers::table
            .select(ServerModelMini::as_select())
            .load(&mut db.pool.get().await.unwrap())
            .await
            .unwrap();

        let semaphore = Arc::new(Semaphore::new(5));

        let handles: Vec<_> = servers
            .into_iter()
            .map(|value| {
                let permit = semaphore.clone().acquire_owned();
                let th_db = db.clone();

                tokio::spawn(async move {
                    let _permit = permit.await;
                    update_server(value, th_db).await;
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        println!("Updating: {}", "DONE".red());
        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}

async fn update_server(server: ServerModelMini, db: Arc<DatabaseWrapper>) {
    let status = match timeout(
        Duration::from_secs(10),
        get_status(&server.ip, server.port as u16),
    )
    .await
    {
        Ok(Ok(b)) => b,
        _ => {
            diesel::update(schema::servers::table)
                .filter(schema::servers::id.eq(&server.id))
                .set(schema::servers::was_online.eq(false))
                .execute(&mut db.pool.get().await.unwrap())
                .await
                .unwrap();
            return;
        }
    };

    let data_insert = DataInsert {
        server_id: server.id,
        online: status.players.online as i32,
        max: status.players.max as i32,
        players: &json!(
            status
                .players
                .sample
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.name)
                .collect::<Vec<String>>()
        ),
    };
    let mut conn = db.pool.get().await.unwrap();

    insert_into(schema::data::table)
        .values(data_insert)
        .execute(&mut conn)
        .await
        .unwrap();

    let players_list = schema::data::table
        .filter(schema::data::server_id.eq(server.id))
        .select(DataModelMini::as_select())
        .load(&mut conn)
        .await
        .unwrap();

    let unique_players = players_list
        .iter()
        .filter_map(|t| t.players.as_array())
        .flatten()
        .filter_map(|t| t.as_str())
        .collect::<HashSet<_>>()
        .len() as i32;

    let server_change = ServerUpdate {
        description: &status.description,
        updated: Utc::now(),
        was_online: true,
        unique_players,
    };

    diesel::update(schema::servers::table)
        .filter(schema::servers::id.eq(server.id))
        .set(server_change)
        .execute(&mut conn)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    colored::control::set_override(true);

    let now = Local::now();
    let time_string = now.format("%Y-%m-%d %H:%M:%S").to_string();

    println!("[{}] Minecarft Lookup | Started", time_string);

    let threads: i32 = env::var("THREADS")
        .unwrap_or("150".to_string())
        .parse()
        .unwrap();

    println!("Threads: {}", threads);

    let db = Arc::new(DatabaseWrapper::establish());
    println!("[+] Connection to database established");

    let count: i64 = schema::servers::table
        .select(diesel::dsl::count(schema::servers::id))
        .first(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    println!("Servers in db: {}", count);

    let updater_thread = tokio::spawn(updater(db.clone()));
    let only_update: bool = env::var("ONLY_UPDATE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);

    println!("Only update: {:?}", only_update);

    if !only_update {
        let mut workers = vec![];

        for _ in 0..threads {
            workers.push(tokio::spawn(worker(db.clone())));
        }

        println!("[+] All threads started");

        for task in workers {
            let _ = task.await;
        }
    }

    updater_thread.await.unwrap();
}
