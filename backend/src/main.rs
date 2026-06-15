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

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../db_schema/migrations");

mod auth;
mod database;
mod events;
mod html;
mod persistence;
mod registry;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).compact().init();

    let config = db_schema::config::Config::load().expect("Failed to load config.toml");
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
        *BACKEND_SECRET.lock().unwrap() =
            match backend_cfg.jwt_secret.filter(|s| !s.is_empty()) {
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
        if token.is_none() {
            tracing::warn!(
                "No [backend].worker_token configured — workers can connect without \
                 authentication. Set one for production."
            );
        }
        *WORKER_TOKEN.lock().unwrap() = token;
    }

    let state = Arc::new(AppState {
        db,
        registry: Arc::new(WorkerRegistry::default()),
        events: Arc::new(crate::events::ServerEvents::default()),
    });

    let api = ApiServer::new(ApiService {
        state: state.clone(),
    });
    let worker = WorkerControlServer::new(WorkerService {
        state: state.clone(),
    });

    // `accept_http1(true)` + `GrpcWebLayer` lets the browser talk gRPC-web while
    // native gRPC (workers, HTTP/2) still passes through unchanged.
    let mut builder = Server::builder().accept_http1(true);

    if let (Some(cert_path), Some(key_path)) = (&backend_cfg.tls_cert, &backend_cfg.tls_key) {
        let cert = std::fs::read(cert_path)?;
        let key = std::fs::read(key_path)?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        builder = builder.tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))?;
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
