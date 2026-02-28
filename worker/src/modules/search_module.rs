use std::{net::IpAddr, sync::Arc, time::Duration};

use chrono::Utc;
use db_schema::{
    models::{
        player_count_snapshots::SnapshotInsert,
        players::PlayerInsert,
        servers::{ServerInsert, ServerModel},
    },
    schema,
};
use diesel::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use tokio::{net::TcpStream, sync::watch, time::timeout};
use tracing::{debug, info};
use worker::{description_to_str, generate_random_ip};

use crate::{
    database::DatabaseWrapper,
    server_actions::{with_connection::get_extra_data, without_connection::get_status},
};

pub async fn check_server(ip: &IpAddr, port: u16) -> anyhow::Result<TcpStream> {
    let addr = format!("{}:{}", ip, port);
    Ok(timeout(Duration::from_millis(750), TcpStream::connect(&addr)).await??)
}

pub async fn search_thread(db: Arc<DatabaseWrapper>, mut pause_watcher: watch::Receiver<bool>) {
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

pub async fn handle_valid_ip(
    ip: &IpAddr,
    port: u16,
    db: Arc<DatabaseWrapper>,
    tcp_stream: Option<TcpStream>,
) -> anyhow::Result<()> {
    let (status, ping) = get_status(&format!("{}", ip), port, tcp_stream).await?;

    let extra_data =
        get_extra_data(format!("{}", ip), port, status.version.protocol as i32).await?;

    let is_forge = status.forge_data.is_some() || status.modinfo.is_some();

    let favicon_ref = status.favicon.as_deref();

    let server_insert = ServerInsert {
        ip: &format!("{}", ip),
        port: port as i32,
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        is_online_mode: extra_data.is_online_mode,
        disconnect_reason: extra_data.disconnect_reason,
        is_forge,
        favicon: favicon_ref,
        ping,
    };

    let mut conn = db.pool.get().await?;

    let server: ServerModel = insert_into(schema::servers::table)
        .values(&server_insert)
        .on_conflict(schema::servers::ip)
        .do_update()
        .set((
            schema::servers::updated_at.eq(Utc::now()),
            schema::servers::is_online.eq(true),
            schema::servers::favicon.eq(favicon_ref),
        ))
        .returning(ServerModel::as_returning())
        .get_result(&mut conn)
        .await?;

    let snapshot_insert = SnapshotInsert {
        server_id: server.id,
        players_online: status.players.online as i32,
        players_max: status.players.max as i32,
    };

    insert_into(schema::player_count_snapshots::table)
        .values(snapshot_insert)
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
        licensed = extra_data.is_online_mode,
        desc = %description_to_str(status.description).unwrap_or_default(),
        has_favicon = status.favicon.is_some(),
        "New server detected"
    );
    Ok(())
}
