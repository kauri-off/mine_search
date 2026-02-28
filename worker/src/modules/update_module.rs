use std::{sync::Arc, time::Duration};

use chrono::Utc;
use db_schema::{
    models::{
        player_count_snapshots::SnapshotInsert,
        players::PlayerInsert,
        servers::{ServerExtraUpdate, ServerModelMini, ServerUpdate},
    },
    schema,
};
use diesel::{dsl::*, pg::Pg, prelude::*, sql_types::Bool};
use diesel_async::RunQueryDsl;
use tokio::{
    sync::{Semaphore, watch},
    time::timeout,
};
use tracing::{debug, error, info};

use crate::{
    database::DatabaseWrapper,
    modules::external_module::process_external_targets,
    server_actions::{with_connection::get_extra_data, without_connection::get_status},
};

pub async fn updater(
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

        if let Err(e) = process_external_targets(db.clone()).await {
            error!(target: "updater", "Error processing external IPs: {}", e);
        }

        info!(target: "updater", "Starting update cycle");

        let spoofable_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
            match only_update_spoofable {
                true => Box::new(schema::servers::is_spoofable.assume_not_null().eq(true)),
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

pub async fn update_server(
    server: ServerModelMini,
    db: Arc<DatabaseWrapper>,
    with_connection: bool,
) {
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
                .set(schema::servers::is_online.eq(false))
                .execute(&mut db.pool.get().await.unwrap())
                .await
                .unwrap();

            return;
        }
    };

    let snapshot_insert = SnapshotInsert {
        server_id: server.id,
        players_online: status.players.online as i32,
        players_max: status.players.max as i32,
    };

    let mut conn = db.pool.get().await.unwrap();

    insert_into(schema::player_count_snapshots::table)
        .values(snapshot_insert)
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

    let is_forge = status.forge_data.is_some() || status.modinfo.is_some();
    let favicon_ref = status.favicon.as_deref();

    let server_change = ServerUpdate {
        version_name: &status.version.name,
        protocol: status.version.protocol as i32,
        description: &status.description,
        updated_at: Utc::now(),
        is_online: true,
        is_forge,
        favicon: favicon_ref,
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
                    is_online_mode: extra_data.is_online_mode,
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
