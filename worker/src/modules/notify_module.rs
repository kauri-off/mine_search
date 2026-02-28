use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use db_schema::{
    models::{ping_requests::ServerPingModel, scan_targets::TargetModel, servers::ServerModelMini},
    schema::{ping_requests, scan_targets, servers},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::Semaphore;
use tracing::error;

use crate::{
    database::DatabaseWrapper,
    modules::{search_module::handle_valid_ip, update_module::update_server},
};

pub async fn notify_listener(db: Arc<DatabaseWrapper>) {
    loop {
        if let Err(e) = notify_batch(&db).await {
            error!(target: "notify_listener", "Batch failed: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn notify_batch(db: &Arc<DatabaseWrapper>) -> anyhow::Result<()> {
    let mut conn = db.pool.get().await?;

    let deleted_pings: Vec<ServerPingModel> = diesel::delete(ping_requests::table)
        .returning(ServerPingModel::as_returning())
        .get_results(&mut conn)
        .await?;

    let server_ids: Vec<i32> = deleted_pings.iter().map(|p| p.server_id).collect();

    let server_map: HashMap<i32, ServerModelMini> = servers::table
        .filter(servers::id.eq_any(&server_ids))
        .select(ServerModelMini::as_select())
        .load(&mut conn)
        .await?
        .into_iter()
        .map(|s: ServerModelMini| (s.id, s))
        .collect();

    let servers: Vec<(ServerModelMini, ServerPingModel)> = deleted_pings
        .into_iter()
        .filter_map(|ping| server_map.get(&ping.server_id).cloned().map(|s| (s, ping)))
        .collect();

    let targets: Vec<TargetModel> = diesel::delete(scan_targets::table)
        .filter(scan_targets::quick.eq(true))
        .returning(TargetModel::as_returning())
        .get_results(&mut conn)
        .await?;

    drop(conn);

    let semaphore = Arc::new(Semaphore::new(50));

    let mut handles: Vec<_> = servers
        .into_iter()
        .map(|(server, server_ping)| {
            let permit = semaphore.clone().acquire_owned();
            let th_db = db.clone();

            tokio::spawn(async move {
                let _permit = permit.await;
                let _ = update_server(server, th_db, server_ping.with_connection).await;
            })
        })
        .collect();

    handles.extend(targets.into_iter().map(|t| {
        let permit = semaphore.clone().acquire_owned();
        let th_db = db.clone();

        tokio::spawn(async move {
            let _permit = permit.await;
            match t.ip.parse::<IpAddr>() {
                Ok(ip) => {
                    let _ = handle_valid_ip(&ip, t.port as u16, th_db, None).await;
                }
                Err(e) => {
                    error!(target: "notify_listener", "Invalid IP in DB: {} - {}", t.ip, e);
                }
            }
        })
    }));

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
