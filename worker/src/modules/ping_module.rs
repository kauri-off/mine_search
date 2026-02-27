use std::{sync::Arc, time::Duration};

use db_schema::{
    models::{server_ping::ServerPingModel, servers::ServerModelMini},
    schema,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::Semaphore;

use crate::{database::DatabaseWrapper, modules::update_module::update_server};

pub async fn server_ping_listener(db: Arc<DatabaseWrapper>) {
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
