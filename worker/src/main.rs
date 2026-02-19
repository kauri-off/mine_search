use std::{collections::HashSet, env, net::IpAddr, sync::Arc, time::Duration};

use chrono::Utc;
use database::DatabaseWrapper;
use diesel::{dsl::insert_into, prelude::*};
use diesel_async::RunQueryDsl;
use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use server_actions::{with_connection::get_extra_data, without_connection::get_status};
use tokio::{
    net::TcpStream,
    sync::{Semaphore, watch},
    time::timeout,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use worker::{check_server, description_to_str, generate_random_ip};

use db_schema::{
    models::{
        data::{DataInsert, DataModelMini},
        ip::IpModel,
        servers::{ServerExtraUpdate, ServerInsert, ServerModel, ServerModelMini, ServerUpdate},
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
    tcp_stream: Option<TcpStream>,
) -> anyhow::Result<()> {
    let status = get_status(&format!("{}", ip), port, tcp_stream).await?;

    let extra_data =
        get_extra_data(format!("{}", ip), port, status.version.protocol as i32).await?;

    let server_insert = ServerInsert {
        ip: &format!("{}", ip),
        port: port as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        license: extra_data.license,
        disconnect_reason: extra_data.disconnect_reason,
        unique_players: status.players.online as i32,
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

    info!(
        target: "server_found",
        ip = %ip,
        port = port,
        version = %status.version.name,
        online = status.players.online,
        max = status.players.max,
        license = extra_data.license,
        desc = %description_to_str(status.description).unwrap_or_default(),
        "New server detected"
    );
    Ok(())
}

async fn worker(db: Arc<DatabaseWrapper>, mut pause_watcher: watch::Receiver<bool>) {
    let mut rng = ChaCha8Rng::try_from_rng(&mut SysRng).unwrap();

    loop {
        if !*pause_watcher.borrow() {
            let _ = pause_watcher.changed().await;
            continue;
        }

        let ip = IpAddr::V4(generate_random_ip(&mut rng));
        const PORT: u16 = 25565;

        if let Ok(tcp_stream) = check_server(&ip, PORT).await {
            debug!("Potential server found at {}:{}", ip, PORT);

            let res = timeout(
                Duration::from_secs(10),
                handle_valid_ip(&ip, PORT, db.clone(), Some(tcp_stream)),
            )
            .await;

            match res {
                Ok(Ok(_)) => info!("Successfully processed server {}:{}", ip, PORT),
                Ok(Err(e)) => error!("Failed to process server {}:{} | Error: {}", ip, PORT, e),
                Err(_) => warn!("Timeout processing server {}:{}", ip, PORT),
            }
        }
    }
}

async fn updater(db: Arc<DatabaseWrapper>, with_connection: bool, pause_tx: watch::Sender<bool>) {
    loop {
        info!(target: "updater", "Stopping workers...");
        let _ = pause_tx.send(false);
        tokio::time::sleep(Duration::from_secs(20)).await;
        info!(target: "updater", "Starting update cycle...");

        let servers: Vec<ServerModelMini> = schema::servers::table
            .select(ServerModelMini::as_select())
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
                    update_server(value, th_db, with_connection).await;
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        info!(target: "updater", "Update cycle finished. Resuming workers.");
        let _ = pause_tx.send(true);
        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}

async fn update_server(server: ServerModelMini, db: Arc<DatabaseWrapper>, with_connection: bool) {
    let status = match timeout(
        Duration::from_secs(10),
        get_status(&server.ip, server.port as u16, None),
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

    if with_connection {
        if let Ok(extra_data) = get_extra_data(
            format!("{}", server.ip),
            server.port as u16,
            status.version.protocol as i32,
        )
        .await
        {
            let server_extra_change = ServerExtraUpdate {
                license: extra_data.license,
                disconnect_reason: extra_data.disconnect_reason,
            };

            diesel::update(schema::servers::table)
                .filter(schema::servers::id.eq(server.id))
                .set(server_extra_change)
                .execute(&mut conn)
                .await
                .unwrap();
        }
    }
}

async fn handle_db_ips(db: Arc<DatabaseWrapper>) {
    loop {
        let mut conn = db.pool.get().await.unwrap();

        let ips: Vec<IpModel> = crate::schema::ips::table
            .select(IpModel::as_select())
            .load(&mut conn)
            .await
            .unwrap();

        diesel::delete(crate::schema::ips::table)
            .execute(&mut conn)
            .await
            .unwrap();

        let tasks: Vec<_> = ips
            .into_iter()
            .map(|ip_model| {
                let db = db.clone();
                tokio::spawn(async move {
                    let ip: IpAddr = match ip_model.ip.parse() {
                        Ok(ip) => ip,
                        Err(e) => {
                            error!("Failed to parse IP '{}': {e}", ip_model.ip);
                            return;
                        }
                    };
                    let port = ip_model.port as u16;

                    if let Err(e) = handle_valid_ip(&ip, port, db, None).await {
                        error!("Failed to handle IP {ip}:{port}: {e}");
                    }
                })
            })
            .collect();

        for task in tasks {
            let _ = task.await;
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
                .add_directive("tokio_postgres=warn".parse().unwrap())
                .add_directive("diesel=warn".parse().unwrap()),
        )
        .init();

    info!("Minecraft Lookup started");

    let threads: i32 = env::var("THREADS")
        .unwrap_or("150".to_string())
        .parse()
        .unwrap();

    info!("Threads: {}", threads);

    let db = Arc::new(DatabaseWrapper::establish());
    debug!("Connection to database established");

    let count: i64 = schema::servers::table
        .select(diesel::dsl::count(schema::servers::id))
        .first(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    debug!("Servers in db: {}", count);

    let (tx, rx) = watch::channel(true);
    let update_with_connection: bool = env::var("UPDATE_WITH_CONNECTION")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let updater_thread = tokio::spawn(updater(db.clone(), update_with_connection, tx));

    let db_updater_thread = tokio::spawn(handle_db_ips(db.clone()));

    let only_update: bool = env::var("ONLY_UPDATE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);

    info!("Only update: {:?}", only_update);

    if !only_update {
        let mut workers = vec![];

        for _ in 0..threads {
            workers.push(tokio::spawn(worker(db.clone(), rx.clone())));
        }

        info!("All threads started");

        for task in workers {
            let _ = task.await;
        }
    }

    updater_thread.await.unwrap();
    db_updater_thread.await.unwrap()
}
