use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod engine;
mod packets;
mod report;
mod server_actions;
mod sink;

#[cfg(feature = "diesel")]
mod diesel_backend;
#[cfg(feature = "grpc")]
mod grpc_backend;

#[cfg(not(any(feature = "grpc", feature = "diesel")))]
compile_error!("enable exactly one of the `grpc` (default) or `diesel` features");

#[tokio::main]
async fn main() {
    let config = config::Config::load().expect("Failed to load worker config (worker.toml)");
    let worker_cfg = config
        .worker
        .clone()
        .expect("Missing [worker] section in worker.toml");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::new(worker_cfg.log_level.as_deref().unwrap_or("info"))
                .add_directive("tokio_postgres=warn".parse().unwrap())
                .add_directive("diesel=warn".parse().unwrap())
                .add_directive("h2=warn".parse().unwrap()),
        )
        .init();

    tracing::info!("mine_search worker starting");

    // The `grpc` feature (default) wins if both happen to be enabled.
    #[cfg(feature = "grpc")]
    {
        let worker_id = resolve_worker_id(&worker_cfg);
        tracing::info!("worker id: {worker_id}");
        // Path the worker rewrites when retuned from the UI, so edits persist.
        let config_path = config::config_path();
        loop {
            match grpc_backend::run(worker_cfg.clone(), worker_id.clone(), config_path.clone()).await
            {
                Ok(()) => {
                    tracing::info!("worker shut down cleanly");
                    break;
                }
                Err(e) => {
                    tracing::warn!("session ended: {e}; reconnecting in 5s");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    #[cfg(all(feature = "diesel", not(feature = "grpc")))]
    {
        use std::sync::Arc;

        let db_url = config
            .database
            .as_ref()
            .expect("Missing [database] section (required in diesel mode)")
            .url
            .clone();

        diesel_backend::run_migrations(&db_url);
        let db = diesel_backend::Database::establish(&db_url);
        let sink = Arc::new(diesel_backend::DieselSink { db: db.clone() });
        let targets = Arc::new(diesel_backend::DieselTargetSource { db });
        let engine = engine::Engine::new(sink, targets, sink::RuntimeConfig::from(&worker_cfg));
        engine.start();
        tracing::info!("worker running (diesel mode)");
        std::future::pending::<()>().await;
    }
}

#[cfg(feature = "grpc")]
fn resolve_worker_id(cfg: &config::WorkerConfig) -> String {
    if let Some(id) = cfg.id.clone().filter(|s| !s.is_empty()) {
        return id;
    }
    // Persist a generated UUID next to the working directory so the identity
    // (and thus operator-pinned config) survives restarts.
    let path = std::path::PathBuf::from("worker_id");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = std::fs::write(&path, &id) {
        tracing::warn!("could not persist worker id: {e}");
    }
    id
}
