use std::sync::Arc;

use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use proto::{api::api_server::ApiServer, worker::worker_control_server::WorkerControlServer};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

use crate::{
    auth::{BACKEND_PASSWORD, BACKEND_SECRET, WORKER_TOKEN, generate_random_string},
    database::DatabaseWrapper,
    registry::WorkerRegistry,
    services::{api::ApiService, worker::WorkerService},
    state::AppState,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

mod auth;
mod chat;
mod config;
mod database;
mod events;
mod html;
mod models;
mod persistence;
mod registry;
mod schema;
#[macro_use]
mod server_filters;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = crate::config::Config::load().expect("Failed to load config.toml");
    let backend_cfg = config
        .backend
        .expect("Missing [backend] section in config.toml");
    let database_cfg = config
        .database
        .expect("Missing [database] section in config.toml");

    let addr = backend_cfg.grpc_addr().parse()?;

    // The backend owns the database: run migrations on startup.
    let mut migration_conn = PgConnection::establish(&database_cfg.url)
        .expect("Failed to connect to database for migrations");
    migration_conn
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    let db = Arc::new(DatabaseWrapper::establish(&database_cfg.url));

    // Install shared secrets used by the auth helpers.
    {
        *BACKEND_PASSWORD.lock().unwrap() = backend_cfg.password.clone();
        *BACKEND_SECRET.lock().unwrap() = match backend_cfg.jwt_secret.filter(|s| !s.is_empty()) {
            Some(secret) => secret,
            None => {
                tracing::warn!(
                    "No [backend].jwt_secret configured — generating a random one. \
                         All sessions will be invalidated on every restart. \
                         Set a stable secret (e.g. `openssl rand -hex 32`) for production."
                );
                generate_random_string(32)
            }
        };
        let token = backend_cfg.worker_token.filter(|s| !s.is_empty());
        let allow_insecure = backend_cfg.allow_insecure_workers.unwrap_or(false);
        if token.is_none() {
            if allow_insecure {
                tracing::warn!(
                    "No [backend].worker_token configured and allow_insecure_workers=true — \
                     accepting UNAUTHENTICATED workers. Do not use this in production."
                );
            } else {
                return Err("No [backend].worker_token configured. Set one, or set \
                            [backend].allow_insecure_workers=true to explicitly permit \
                            unauthenticated workers (dev only)."
                    .into());
            }
        }
        auth::ALLOW_INSECURE_WORKERS.store(allow_insecure, std::sync::atomic::Ordering::Relaxed);
        *WORKER_TOKEN.lock().unwrap() = token;
    }

    // The manual "update stack" action needs both a watchtower URL and token.
    let watchtower = match (
        backend_cfg.watchtower_url.filter(|s| !s.is_empty()),
        backend_cfg.watchtower_token.filter(|s| !s.is_empty()),
    ) {
        (Some(url), Some(token)) => Some(crate::state::WatchtowerConfig { url, token }),
        _ => {
            tracing::info!(
                "No [backend].watchtower_url/watchtower_token configured — the manual \
                 stack-update action is disabled."
            );
            None
        }
    };

    let state = Arc::new(AppState {
        db,
        registry: Arc::new(WorkerRegistry::default()),
        events: Arc::new(crate::events::ServerEvents::default()),
        watchtower,
    });

    // Periodically prune the worker-result idempotency ledger. First tick fires
    // immediately, then hourly.
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match crate::persistence::prune_processed_results(&state.db).await {
                    Ok(n) if n > 0 => tracing::info!("pruned {n} processed_results rows"),
                    Ok(_) => {}
                    Err(e) => tracing::warn!("failed to prune processed_results: {e}"),
                }
            }
        });
    }

    // Explicit message-size caps (tonic's default decode cap is 4 MB; make it
    // explicit and bound the encode side too). Requests/responses here are small
    // now that update targets stream one row at a time.
    const MAX_DECODING: usize = 4 * 1024 * 1024;
    const MAX_ENCODING: usize = 16 * 1024 * 1024;

    let api = ApiServer::new(ApiService {
        state: state.clone(),
    })
    .max_decoding_message_size(MAX_DECODING)
    .max_encoding_message_size(MAX_ENCODING);
    let worker = WorkerControlServer::new(WorkerService {
        state: state.clone(),
    })
    .max_decoding_message_size(MAX_DECODING)
    .max_encoding_message_size(MAX_ENCODING);

    // `accept_http1(true)` + `GrpcWebLayer` lets the browser talk gRPC-web while
    // native gRPC (workers, HTTP/2) still passes through unchanged.
    //
    // Bound per-connection resource use: cap concurrent in-flight requests per
    // connection, and use HTTP/2 keepalive pings to detect and reclaim dead
    // peers (which would otherwise leak long-lived Session streams). A blanket
    // `Server::timeout` is intentionally omitted — it would sever the
    // long-lived bidirectional `Session` and the streaming `FetchUpdateTargets`
    // RPCs; the unary handlers are individually cheap.
    let mut builder = Server::builder()
        .concurrency_limit_per_connection(256)
        .tcp_keepalive(Some(std::time::Duration::from_secs(75)))
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(30)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(20)))
        .accept_http1(true);

    if let (Some(cert_path), Some(key_path)) = (&backend_cfg.tls_cert, &backend_cfg.tls_key) {
        let cert = std::fs::read(cert_path)?;
        let key = std::fs::read(key_path)?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        builder =
            builder.tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))?;
        tracing::info!("TLS enabled");
    }

    tracing::info!("gRPC server listening on {addr}");

    builder
        .layer(GrpcWebLayer::new())
        .add_service(api)
        .add_service(worker)
        .serve(addr)
        .await?;

    Ok(())
}
