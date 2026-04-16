use std::sync::Arc;

use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../db_schema/migrations");

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
    let config = db_schema::config::Config::load().expect("Failed to load config.toml");
    let worker_cfg = config
        .worker
        .expect("Missing [worker] section in config.toml");

    let threads = worker_cfg.threads;
    let search_module = worker_cfg.search_module;
    let update_module = worker_cfg.update_module;
    let update_with_connection = worker_cfg.update_with_connection;
    let only_update_spoofable = worker_cfg.only_update_spoofable;
    let only_update_cracked = worker_cfg.only_update_cracked;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::new(worker_cfg.log_level.as_deref().unwrap_or("info"))
                .add_directive(
                    "tokio_postgres=warn"
                        .parse()
                        .expect("hardcoded tracing directive is valid"),
                )
                .add_directive(
                    "diesel=warn"
                        .parse()
                        .expect("hardcoded tracing directive is valid"),
                ),
        )
        .init();

    info!("mine_search starting");
    info!("Threads: {}", threads);

    let mut migration_conn = PgConnection::establish(&config.database.url)
        .expect("Failed to connect to database for migrations");
    migration_conn
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    let db = Arc::new(DatabaseWrapper::establish(&config.database.url));
    debug!("Connection to database established");

    let count: i64 = schema::servers::table
        .select(diesel::dsl::count(schema::servers::id))
        .first(
            &mut db
                .pool
                .get()
                .await
                .expect("Failed to get DB connection at startup"),
        )
        .await
        .expect("Failed to count servers at startup");

    debug!("Servers in db: {}", count);

    info!("Search module: {:?}", search_module);
    info!("Update module: {:?}", update_module);

    if update_module {
        info!("Update with connection: {:?}", update_with_connection);
        info!("Only update spoofable: {:?}", only_update_spoofable);
        info!("Only update cracked: {:?}", only_update_cracked);
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
            only_update_cracked,
        )));
    }

    tasks.push(tokio::spawn(notify_listener(db.clone())));

    for task in tasks {
        let _ = task.await;
    }
}
