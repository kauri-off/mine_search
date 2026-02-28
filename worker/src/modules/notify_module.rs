use std::{sync::Arc, time::Duration};

use db_schema::{
    models::{ping_requests::ServerPingModel, scan_targets::TargetModel, servers::ServerModelMini},
    schema::{ping_requests, scan_targets, servers},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::Semaphore;

use crate::{
    database::DatabaseWrapper,
    modules::{search_module::handle_valid_ip, update_module::update_server},
};

pub async fn notify_listener(db: Arc<DatabaseWrapper>) {
    loop {
        let mut conn = db.pool.get().await.unwrap();

        let servers: Vec<(ServerModelMini, ServerPingModel)> = ping_requests::table
            .inner_join(servers::table.on(ping_requests::server_id.eq(servers::id)))
            .select((ServerModelMini::as_select(), ServerPingModel::as_select()))
            .load(&mut conn)
            .await
            .unwrap();

        diesel::delete(ping_requests::table)
            .execute(&mut conn)
            .await
            .unwrap();

        let targets: Vec<TargetModel> = scan_targets::table
            .filter(scan_targets::quick.eq(true))
            .select(TargetModel::as_select())
            .load(&mut conn)
            .await
            .unwrap();

        diesel::delete(scan_targets::table)
            .filter(scan_targets::quick.eq(true))
            .execute(&mut conn)
            .await
            .unwrap();

        drop(conn);

        let semaphore = Arc::new(Semaphore::new(50));

        let mut handles: Vec<_> = servers
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

        handles.extend(targets.into_iter().map(|t| {
            let permit = semaphore.clone().acquire_owned();
            let th_db = db.clone();

            tokio::spawn(async move {
                let _permit = permit.await;
                let _ = handle_valid_ip(&t.ip.parse().unwrap(), t.port as u16, th_db, None).await;
            })
        }));

        for handle in handles {
            let _ = handle.await;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
