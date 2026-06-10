use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    #[serde(skip_serializing)]
    pub database: DatabaseConfig,
    #[serde(skip_serializing)]
    pub auth: AuthConfig,
    pub parser: ParserConfig,
    pub filter: FilterConfig,
    pub pusher: PusherConfig,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
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

fn default_interval() -> u64 {
    30
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ParserConfig {
    pub max_concurrent_fetches: usize,
    pub default_user_agent: String,
    pub default_timeout_seconds: u64,
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct FilterConfig {
    pub batch_size: u32,
    pub interval_seconds: u64,
    pub history_hours: u32,
    pub min_history_hours: u32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PusherConfig {
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_base_seconds: u64,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("server.port must be > 0".into());
        }
        if self.database.path.is_empty() {
            return Err("database.path must not be empty".into());
        }
        if self.parser.interval_seconds == 0 {
            return Err("parser.interval_seconds must be > 0".into());
        }
        if self.parser.max_concurrent_fetches == 0 {
            return Err("parser.max_concurrent_fetches must be > 0".into());
        }
        if self.filter.interval_seconds == 0 {
            return Err("filter.interval_seconds must be > 0".into());
        }
        if self.filter.batch_size == 0 {
            return Err("filter.batch_size must be > 0".into());
        }
        if self.pusher.interval_seconds == 0 {
            return Err("pusher.interval_seconds must be > 0".into());
        }
        if self.pusher.max_retries == 0 {
            return Err("pusher.max_retries must be > 0".into());
        }
        Ok(())
    }
}
