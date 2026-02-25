use std::{env, net::IpAddr, sync::Arc, time::Duration};

use chrono::Utc;
use database::DatabaseWrapper;
use diesel::{dsl::insert_into, pg::Pg, prelude::*, sql_types::Bool};
use diesel_async::RunQueryDsl;
use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use server_actions::{with_connection::get_extra_data, without_connection::get_status};
use tokio::{
    net::TcpStream,
    sync::{Semaphore, watch},
    time::timeout,
};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use worker::{check_server, description_to_str, generate_random_ip};

use db_schema::{
    models::{
        data::DataInsert,
        ip::IpModel,
        players::PlayerInsert,
        server_ping::ServerPingModel,
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

    let is_modded = status.forge_data.is_some() || status.is_modded.unwrap_or(false);

    let server_insert = ServerInsert {
        ip: &format!("{}", ip),
        port: port as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        license: extra_data.license,
        disconnect_reason: extra_data.disconnect_reason,
        is_modded,
    };

    let mut conn = db.pool.get().await?;

    let server: ServerModel = insert_into(schema::servers::table)
        .values(&server_insert)
        .on_conflict(schema::servers::ip)
        .do_update()
        .set((
            schema::servers::updated.eq(Utc::now()),
            schema::servers::was_online.eq(true),
        ))
        .returning(ServerModel::as_returning())
        .get_result(&mut conn)
        .await?;

    let data_insert = DataInsert {
        server_id: server.id,
        online: status.players.online as i32,
        max: status.players.max as i32,
    };

    insert_into(schema::data::table)
        .values(data_insert)
        .execute(&mut conn)
        .await?;

    insert_into(schema::players::table)
        .values(
            status
                .players
                .sample
                .unwrap_or_default()
                .iter()
                .map(|t| PlayerInsert {
                    server_id: server.id,
                    name: &t.name,
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
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
                Ok(Ok(_)) => {}
                Ok(Err(e)) => debug!("Failed to process server {}:{} | Error: {}", ip, PORT, e),
                Err(_) => debug!("Timeout processing server {}:{}", ip, PORT),
            }
        }
    }
}

async fn updater(
    db: Arc<DatabaseWrapper>,
    with_connection: bool,
    pause_tx: watch::Sender<bool>,
    search_module: bool,
    only_update_spoofable: bool,
) {
    loop {
        if search_module {
            info!(target: "updater", "Stopping workers");
            let _ = pause_tx.send(false);
            tokio::time::sleep(Duration::from_secs(20)).await;
        }

        if let Err(e) = process_external_ips(db.clone()).await {
            error!(target: "updater", "Error processing external IPs: {}", e);
        }

        info!(target: "updater", "Starting update cycle");

        let spoofable_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
            match only_update_spoofable {
                true => Box::new(schema::servers::spoofable.assume_not_null().eq(true)),
                false => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
            };

        let servers: Vec<ServerModelMini> = schema::servers::table
            .filter(spoofable_filter)
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

        info!(target: "updater", "Update cycle finished");

        if search_module {
            let _ = pause_tx.send(true);
            info!(target: "updater", "Resuming workers");
        }

        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}

async fn process_external_ips(db: Arc<DatabaseWrapper>) -> anyhow::Result<()> {
    let mut conn = db.pool.get().await?;

    let ips: Vec<IpModel> = schema::ips::table
        .select(IpModel::as_select())
        .load(&mut conn)
        .await?;

    if ips.is_empty() {
        return Ok(());
    }

    info!("Processing {} external ips", ips.len());

    diesel::delete(schema::ips::table)
        .execute(&mut conn)
        .await?;

    drop(conn);

    let semaphore = Arc::new(Semaphore::new(50));

    let handles: Vec<_> = ips
        .into_iter()
        .map(|value| {
            let permit = semaphore.clone().acquire_owned();
            let th_db = db.clone();

            tokio::spawn(async move {
                let _permit = permit.await;
                if let Ok(ip) = value.ip.parse() {
                    let port = value.port as u16;
                    let _ = timeout(
                        Duration::from_secs(5),
                        handle_valid_ip(&ip, port, th_db, None),
                    )
                    .await;
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn update_server(server: ServerModelMini, db: Arc<DatabaseWrapper>, with_connection: bool) {
    let status = match timeout(
        Duration::from_secs(10),
        get_status(&server.ip, server.port as u16, None),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Err(_) | Ok(_) => {
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
    };

    let mut conn = db.pool.get().await.unwrap();

    insert_into(schema::data::table)
        .values(data_insert)
        .execute(&mut conn)
        .await
        .unwrap();

    insert_into(schema::players::table)
        .values(
            status
                .players
                .sample
                .unwrap_or_default()
                .iter()
                .map(|t| PlayerInsert {
                    server_id: server.id,
                    name: &t.name,
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await
        .unwrap();

    let is_modded = status.forge_data.is_some() || status.is_modded.unwrap_or(false);

    let server_change = ServerUpdate {
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        updated: Utc::now(),
        was_online: true,
        is_modded,
    };

    diesel::update(schema::servers::table)
        .filter(schema::servers::id.eq(server.id))
        .set(server_change)
        .execute(&mut conn)
        .await
        .unwrap();

    if with_connection {
        match timeout(
            Duration::from_secs(5),
            get_extra_data(
                server.ip.clone(),
                server.port as u16,
                status.version.protocol as i32,
            ),
        )
        .await
        {
            Ok(Ok(extra_data)) => {
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
            Err(_) | Ok(Err(_)) => debug!("Could not get extra data for {}", server.ip),
        }
    }
}

async fn server_ping_listener(db: Arc<DatabaseWrapper>) {
    loop {
        let mut conn = db.pool.get().await.unwrap();

        let servers: Vec<(ServerModelMini, ServerPingModel)> = schema::server_ping::table
            .inner_join(
                schema::servers::table.on(schema::server_ping::server_id.eq(schema::servers::id)),
            )
            .select((ServerModelMini::as_select(), ServerPingModel::as_select()))
            .load(&mut conn)
            .await
            .unwrap();

        diesel::delete(schema::server_ping::table)
            .execute(&mut conn)
            .await
            .unwrap();

        drop(conn);

        let semaphore = Arc::new(Semaphore::new(50));

        let handles: Vec<_> = servers
            .into_iter()
            .map(|(server, server_ping)| {
                let permit = semaphore.clone().acquire_owned();
                let th_db = db.clone();

                tokio::spawn(async move {
                    let _permit = permit.await;
                    update_server(server, th_db, server_ping.with_connection).await;
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
    let threads: i32 = env::var("THREADS")
        .unwrap_or("150".to_string())
        .parse()
        .unwrap();

    let search_module: bool = env::var("SEARCH_MODULE")
        .unwrap_or("true".to_string())
        .parse()
        .unwrap_or(true);

    let update_module: bool = env::var("UPDATE_MODULE")
        .unwrap_or("true".to_string())
        .parse()
        .unwrap_or(true);

    let update_with_connection: bool = env::var("UPDATE_WITH_CONNECTION")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);

    let only_update_spoofable: bool = env::var("ONLY_UPDATE_SPOOFABLE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
                .add_directive("tokio_postgres=warn".parse().unwrap())
                .add_directive("diesel=warn".parse().unwrap()),
        )
        .init();

    info!("mine_search starting");
    info!("Threads: {}", threads);

    let db = Arc::new(DatabaseWrapper::establish());
    debug!("Connection to database established");

    let count: i64 = schema::servers::table
        .select(diesel::dsl::count(schema::servers::id))
        .first(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();

    debug!("Servers in db: {}", count);

    info!("Search module: {:?}", search_module);
    info!("Update module: {:?}", update_module);

    if update_module {
        info!("Only update spoofable: {:?}", only_update_spoofable);
    }

    let (tx, rx) = watch::channel(true);

    let mut tasks = vec![];

    if search_module {
        for _ in 0..threads {
            tasks.push(tokio::spawn(worker(db.clone(), rx.clone())));
        }
        info!("All worker threads started");
    }

    if update_module {
        tasks.push(tokio::spawn(updater(
            db.clone(),
            update_with_connection,
            tx,
            search_module,
            only_update_spoofable,
        )));
    }

    tasks.push(tokio::spawn(server_ping_listener(db.clone())));

    for task in tasks {
        let _ = task.await;
    }
}
