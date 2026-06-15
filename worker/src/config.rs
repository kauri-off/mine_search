//! Worker-local config parsing. Kept independent of `db_schema` so a gRPC-only
//! build (the default) does not pull in diesel/postgres just to read a TOML file.

use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    // Only used in diesel mode.
    #[allow(dead_code)]
    pub database: Option<DatabaseConfig>,
    pub worker: Option<WorkerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    #[allow(dead_code)]
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // gRPC-only fields are unused in diesel-only builds
pub struct WorkerConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    pub update_with_connection: bool,
    pub only_update_spoofable: bool,
    pub only_update_cracked: bool,
    #[serde(default = "default_update_interval")]
    pub update_interval_secs: u32,
    #[serde(default = "default_update_concurrency")]
    pub update_concurrency: u32,
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
