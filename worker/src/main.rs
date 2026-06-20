use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod engine;
mod grpc_backend;
mod outbox;
mod packets;
mod report;
mod server_actions;

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
                .add_directive("h2=warn".parse().unwrap()),
        )
        .init();

    tracing::info!("mine_search worker starting");

    // Path the worker rewrites when retuned from the UI, so edits persist.
    let config_path = config::config_path();
    let worker_id = resolve_worker_id(&worker_cfg, &config_path);
    tracing::info!("worker id: {worker_id}");

    // Durable store of scan results awaiting backend acks; survives reconnects
    // and restarts, so discovered servers are not lost when the link drops.
    let outbox_path = config_path.with_file_name("outbox.log");
    let outbox = std::sync::Arc::new(
        outbox::Outbox::open(&outbox_path).expect("Failed to open outbox log"),
    );

    loop {
        match grpc_backend::run(
            worker_cfg.clone(),
            worker_id.clone(),
            config_path.clone(),
            outbox.clone(),
        )
        .await
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

fn resolve_worker_id(cfg: &config::WorkerConfig, config_path: &std::path::Path) -> String {
    if let Some(id) = cfg.id.clone().filter(|s| !s.is_empty()) {
        return id;
    }
    // Generate a UUID and persist it back into the worker's config file so the
    // identity (and thus operator-pinned config) survives restarts.
    let id = uuid::Uuid::new_v4().to_string();
    grpc_backend::persist_id(config_path, &id);
    id
}
