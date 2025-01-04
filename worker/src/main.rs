use std::{
    env,
    io::{self, ErrorKind},
    net::IpAddr,
    sync::Arc,
    time::Duration,
};

use chrono::{Local, Timelike};
use colored::Colorize;
use database::{DatabaseWrapper, PlayerInsert, ServerInsert, ServerModel, ServerUpdate};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use mine_search::{check_server, description_to_str, generate_random_ip};
use serde_json::json;
use server_actions::{with_connection::get_extra_data, without_connection::get_status};
use tokio::{sync::Semaphore, time::timeout};

use db_schema::schema;

mod conn_wrapper;
mod database;
mod packets;
mod server_actions;

pub async fn handle_valid_ip(ip: &IpAddr, port: u16, db: Arc<DatabaseWrapper>) -> io::Result<()> {
    let status = get_status(&format!("{}", ip), port).await?;

    let extra_data =
        get_extra_data(format!("{}", ip), port, status.version.protocol as i32).await?;

    let server_insert = ServerInsert {
        ip: &format!("{}", ip),
        online: status.players.online as i32,
        max: status.players.max as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        license: extra_data.license,
        white_list: extra_data.white_list,
        description: &json!({
            "payload": status.description
        }),
    };
    let mut conn = db.pool.get().await.unwrap();

    let server: ServerModel = insert_into(schema::servers::dsl::servers)
        .values(server_insert)
        .on_conflict(schema::servers::dsl::ip)
        .do_nothing()
        .returning(ServerModel::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|_| ErrorKind::InvalidInput)?;

    for player in status.players.sample.unwrap_or_default() {
        let player_model = PlayerInsert {
            uuid: &player.id,
            name: &player.name,
            server_id: server.id,
        };

        insert_into(schema::players::dsl::players)
            .values(&player_model)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .unwrap();
    }

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
    loop {
        let ip = IpAddr::V4(generate_random_ip());

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

        let servers: Vec<ServerModel> = schema::servers::dsl::servers
            .select(ServerModel::as_select())
            .load(&mut db.pool.get().await.unwrap())
            .await
            .unwrap();

        let semaphore = Arc::new(Semaphore::new(50));

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

async fn update_server(server: ServerModel, db: Arc<DatabaseWrapper>) {
    let status = match timeout(Duration::from_secs(2), get_status(&server.ip, 25565)).await {
        Ok(t) => match t {
            Ok(b) => b,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let server_update = ServerUpdate {
        online: status.players.online as i32,
        max: status.players.max as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &json!({
            "payload": status.description,
        }),
    };
    let mut conn = db.pool.get().await.unwrap();

    diesel::update(schema::servers::dsl::servers)
        .filter(schema::servers::dsl::ip.eq(&server.ip))
        .set((
            server_update,
            schema::servers::dsl::last_seen
                .eq(Local::now().naive_local().with_nanosecond(0).unwrap()),
        ))
        .execute(&mut conn)
        .await
        .unwrap();

    for player in status.players.sample.unwrap_or_default() {
        let player_model = PlayerInsert {
            uuid: &player.id,
            name: &player.name,
            server_id: server.id,
        };

        insert_into(schema::players::dsl::players)
            .values(&player_model)
            .on_conflict((schema::players::dsl::name, schema::players::dsl::server_id))
            .do_update()
            .set(
                schema::players::dsl::last_seen
                    .eq(Local::now().naive_local().with_nanosecond(0).unwrap()),
            )
            .execute(&mut conn)
            .await
            .unwrap();
    }
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

    let count: i64 = schema::servers::dsl::servers
        .select(diesel::dsl::count(schema::servers::dsl::id))
        .first(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    println!("Servers in db: {}", count);

    let updater_thread = tokio::spawn(updater(db.clone()));
    let only_update: bool = env::var("ONLY_UPDATE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);

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
