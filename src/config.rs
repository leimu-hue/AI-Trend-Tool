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
    #[serde(default)]
    pub logging: LoggingConfig,
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

// ── LoggingConfig default functions ──

fn default_log_dir() -> String {
    "./logs".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_files() -> u32 {
    30
}

fn default_max_days() -> u32 {
    30
}

fn default_max_total_size_mb() -> u32 {
    500
}

fn default_console_output() -> bool {
    true
}

// ── LoggingConfig ──

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_dir")]
    pub dir: String,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_max_files")]
    pub max_files: u32,
    #[serde(default = "default_max_days")]
    pub max_days: u32,
    #[serde(default = "default_max_total_size_mb")]
    pub max_total_size_mb: u32,
    #[serde(default = "default_console_output")]
    pub console_output: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            dir: default_log_dir(),
            level: default_log_level(),
            max_files: default_max_files(),
            max_days: default_max_days(),
            max_total_size_mb: default_max_total_size_mb(),
            console_output: default_console_output(),
        }
    }
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

fn default_retry_max_seconds() -> u64 {
    3600
}

fn default_stale_timeout_minutes() -> u64 {
    10
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PusherConfig {
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_base_seconds: u64,
    /// Maximum backoff delay in seconds (default 3600 = 1 hour)
    #[serde(default = "default_retry_max_seconds")]
    pub retry_max_seconds: u64,
    /// Minutes before a stuck processing record is recovered (default 10)
    #[serde(default = "default_stale_timeout_minutes")]
    pub stale_timeout_minutes: u64,
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
        if self.pusher.retry_max_seconds == 0 {
            return Err("pusher.retry_max_seconds must be > 0".into());
        }
        if self.pusher.stale_timeout_minutes == 0 {
            return Err("pusher.stale_timeout_minutes must be > 0".into());
        }
        // ── Logging validation ──
        if self.logging.dir.is_empty() {
            return Err("logging.dir must not be empty".into());
        }
        {
            let valid_levels = ["trace", "debug", "info", "warn", "error"];
            if !valid_levels.contains(&self.logging.level.as_str()) {
                return Err(format!("logging.level must be one of: {:?}", valid_levels).into());
            }
        }
        if self.logging.max_files == 0 {
            return Err("logging.max_files must be > 0".into());
        }
        if self.logging.max_days == 0 {
            return Err("logging.max_days must be > 0".into());
        }
        if self.logging.max_total_size_mb == 0 {
            return Err("logging.max_total_size_mb must be > 0".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 3000,
            },
            database: DatabaseConfig {
                path: "data/hotspot.db".into(),
            },
            auth: AuthConfig {
                initial_token: None,
            },
            parser: ParserConfig {
                max_concurrent_fetches: 10,
                default_user_agent: "Test/1.0".into(),
                default_timeout_seconds: 30,
                interval_seconds: 30,
            },
            filter: FilterConfig {
                batch_size: 100,
                interval_seconds: 300,
                history_hours: 24,
                min_history_hours: 2,
            },
            pusher: PusherConfig {
                interval_seconds: 10,
                max_retries: 3,
                retry_base_seconds: 60,
                retry_max_seconds: 3600,
                stale_timeout_minutes: 10,
            },
            logging: LoggingConfig::default(),
        }
    }

    #[test]
    fn valid_config_passes() {
        let config = valid_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn port_zero_rejected() {
        let mut config = valid_config();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_db_path_rejected() {
        let mut config = valid_config();
        config.database.path = "".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn zero_max_concurrent_fetches_rejected() {
        let mut config = valid_config();
        config.parser.max_concurrent_fetches = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn zero_batch_size_rejected() {
        let mut config = valid_config();
        config.filter.batch_size = 0;
        assert!(config.validate().is_err());
    }
}
