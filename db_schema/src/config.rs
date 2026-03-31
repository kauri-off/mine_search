use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
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
}

#[derive(Debug, Deserialize)]
pub struct WorkerConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    pub update_with_connection: bool,
    pub only_update_spoofable: bool,
    pub only_update_cracked: bool,
    pub log_level: Option<String>,
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
