use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    // Only required by the backend and by workers built with the `diesel`
    // feature. gRPC workers never talk to the database directly, so they may
    // omit this section entirely.
    pub database: Option<DatabaseConfig>,
    pub backend: Option<BackendConfig>,
    pub worker: Option<WorkerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub password: String,
    pub jwt_secret: Option<String>,
    /// Address the tonic server binds to (serves both the frontend gRPC-web API
    /// and the worker control plane). Defaults to `0.0.0.0:3000`.
    pub grpc_addr: Option<String>,
    /// Shared secret a worker must present (Bearer) to connect. When unset, the
    /// backend accepts unauthenticated workers and logs a warning.
    pub worker_token: Option<String>,
    /// Optional TLS for the gRPC server. Both must be set to enable TLS;
    /// otherwise the server is plaintext (intended to sit behind nginx).
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

impl BackendConfig {
    pub fn grpc_addr(&self) -> String {
        self.grpc_addr
            .clone()
            .unwrap_or_else(|| "0.0.0.0:3000".to_string())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    pub update_with_connection: bool,
    pub only_update_spoofable: bool,
    pub only_update_cracked: bool,
    pub log_level: Option<String>,

    // ----- gRPC mode (default build) -----
    /// Backend endpoint the worker dials, e.g. `http://127.0.0.1:3000` or
    /// `https://example.com:443`. Required when built with the `grpc` feature.
    pub backend_url: Option<String>,
    /// Shared secret presented to the backend. Must match `[backend].worker_token`.
    pub token: Option<String>,
    /// Stable worker identity. If unset, a UUID is generated and persisted to
    /// `worker_id` next to the config so it survives restarts/reconnects.
    pub id: Option<String>,
    /// Human-friendly name shown in the management UI.
    pub name: Option<String>,
    /// PEM CA bundle to trust for the backend's TLS cert (custom CA / self-signed).
    pub tls_ca: Option<String>,
    /// Skip TLS certificate verification (dev only).
    pub insecure: Option<bool>,
}

impl Config {
    /// Load config from the path given by --config <path>, CONFIG_PATH env var,
    /// or default "config.toml" in the working directory.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path();
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", path, e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {:?}: {}", path, e))?;
        Ok(config)
    }
}

fn config_path() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--config") {
        if let Some(val) = args.get(pos + 1) {
            return PathBuf::from(val);
        }
    }
    if let Ok(val) = env::var("CONFIG_PATH") {
        return PathBuf::from(val);
    }
    PathBuf::from("config.toml")
}
