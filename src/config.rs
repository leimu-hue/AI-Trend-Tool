use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub parser: ParserConfig,
    pub filter: FilterConfig,
    pub pusher: PusherConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub initial_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParserConfig {
    pub max_concurrent_fetches: usize,
    pub default_user_agent: String,
    pub default_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilterConfig {
    pub batch_size: u32,
    pub interval_seconds: u64,
    pub history_hours: u32,
    pub min_history_hours: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PusherConfig {
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_base_seconds: u64,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
