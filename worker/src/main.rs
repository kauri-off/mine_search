use std::{env, sync::Arc};

use db_schema::schema;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::watch;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    database::DatabaseWrapper,
    modules::{
        notify_module::notify_listener, search_module::search_thread, update_module::updater,
    },
};

mod database;
mod modules;
mod packets;
mod server_actions;

#[tokio::main]
async fn main() {
    let threads: i32 = env::var("THREADS")
        .unwrap_or("150".to_string())
        .parse()
        .expect("THREADS env var must be a valid i32");

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
                .add_directive("tokio_postgres=warn".parse().expect("hardcoded tracing directive is valid"))
                .add_directive("diesel=warn".parse().expect("hardcoded tracing directive is valid")),
        )
        .init();

    info!("mine_search starting");
    info!("Threads: {}", threads);

    let db = Arc::new(DatabaseWrapper::establish());
    debug!("Connection to database established");

    let count: i64 = schema::servers::table
        .select(diesel::dsl::count(schema::servers::id))
        .first(&mut db.pool.get().await.expect("Failed to get DB connection at startup"))
        .await
        .expect("Failed to count servers at startup");

    debug!("Servers in db: {}", count);

    info!("Search module: {:?}", search_module);
    info!("Update module: {:?}", update_module);

    if update_module {
        info!("Update with connection: {:?}", update_with_connection);
        info!("Only update spoofable: {:?}", only_update_spoofable);
    }

    let (tx, rx) = watch::channel(true);

    let mut tasks = vec![];

    if search_module {
        for _ in 0..threads {
            tasks.push(tokio::spawn(search_thread(db.clone(), rx.clone())));
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

    tasks.push(tokio::spawn(notify_listener(db.clone())));

    for task in tasks {
        let _ = task.await;
    }
}
