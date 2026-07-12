//! Worker-local config parsing. The worker never talks to a database directly
//! (it streams to the backend over gRPC), so this only needs to read a TOML file.

use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub worker: Option<WorkerConfig>,
}

/// Optional server-property filters (mirrors `worker.ServerFilter`), deserialized
/// from the `[worker.update_filter]` / `[worker.search_filter]` tables. Every
/// field defaults to `None` = "no constraint".
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ServerFilter {
    pub online: Option<bool>,
    pub licensed: Option<bool>,
    pub checked: Option<bool>,
    pub crashed: Option<bool>,
    pub requires_mods: Option<bool>,
    pub has_players: Option<bool>,
    pub has_none_players: Option<bool>,
    pub join_status: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    #[serde(default)]
    pub threads: i32,
    #[serde(default)]
    pub search_module: bool,
    #[serde(default)]
    pub update_module: bool,
    #[serde(default)]
    pub update_with_connection: bool,
    #[serde(default = "default_update_interval")]
    pub update_interval_secs: u32,
    #[serde(default = "default_update_concurrency")]
    pub update_concurrency: u32,
    // Which existing servers the update cycle re-probes.
    #[serde(default)]
    pub update_filter: ServerFilter,
    // Acceptance filter applied to freshly discovered servers before reporting.
    #[serde(default)]
    pub search_filter: ServerFilter,
    pub log_level: Option<String>,

    // gRPC mode
    pub backend_url: Option<String>,
    pub token: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub tls_ca: Option<String>,
    pub insecure: Option<bool>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path();
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", path, e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {:?}: {}", path, e))?;
        Ok(config)
    }
}

fn default_update_interval() -> u32 {
    600
}

fn default_update_concurrency() -> u32 {
    50
}

/// Path to the worker's config file: `--config <path>`, `CONFIG_PATH`, or the
/// default `worker.toml`. The worker owns this file (it rewrites the live-tunable
/// `[worker]` keys when retuned from the UI), so it is separate from the backend's
/// `config.toml`.
pub fn config_path() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--config") {
        if let Some(val) = args.get(pos + 1) {
            return PathBuf::from(val);
        }
    }
    if let Ok(val) = env::var("CONFIG_PATH") {
        return PathBuf::from(val);
    }
    PathBuf::from("worker.toml")
}
