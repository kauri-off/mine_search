use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: Option<DatabaseConfig>,
    pub backend: Option<BackendConfig>,
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
    /// Base URL of the watchtower HTTP API (e.g. `http://watchtower:8080`). When
    /// unset, the manual "update stack" action is disabled.
    pub watchtower_url: Option<String>,
    /// Bearer token watchtower expects on its HTTP API (`WATCHTOWER_HTTP_API_TOKEN`).
    pub watchtower_token: Option<String>,
}

impl BackendConfig {
    pub fn grpc_addr(&self) -> String {
        self.grpc_addr
            .clone()
            .unwrap_or_else(|| "0.0.0.0:3000".to_string())
    }
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
