use std::{sync::Arc, time::Duration};

use db_schema::{models::ip::IpModel, schema};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::{sync::Semaphore, time::timeout};
use tracing::info;

use crate::{database::DatabaseWrapper, modules::search_module::handle_valid_ip};

pub async fn process_external_ips(db: Arc<DatabaseWrapper>) -> anyhow::Result<()> {
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
