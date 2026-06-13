# 12 - Backend File Logging

## Goal

Add structured file-based logging to the backend so that errors and warnings are
persisted to `./logs/` with automatic rotation, max file count, max total size,
and retention-day enforcement.

---

## Current State

| Aspect | Detail |
|--------|--------|
| Logging crate | `tracing` 0.1 + `tracing-subscriber` 0.3 |
| Output | **stdout only** |
| Init location | `src/main.rs:95` — `tracing_subscriber::fmt().with_env_filter("info").init()` |
| Existing log calls | ~25 `tracing::error!` / `tracing::warn!` in `services/parser.rs`, `services/filter.rs`, `services/pusher.rs`, `error.rs` |
| Rotation / retention | None |

---

## Design Decisions

1. **Crate**: `tracing-appender` 0.2 — official companion to `tracing-subscriber`, provides rolling file appenders.
2. **Rotation strategy**: Daily rotation via `RollingFileAppender::builder().rotation_daily()`. Each day produces a file named `app-YYYY-MM-DD.log`.
3. **Dual output**: Keep console (stdout) logging AND add file logging simultaneously using `tracing_subscriber` layer composition (`Registry` + multiple `fmt::Layer`).
4. **Retention enforcement**: `tracing-appender` natively supports `max_log_files`. For day-based and size-based cleanup, a background async task runs on startup and periodically (every 6 hours) to:
   - Delete log files older than `max_days`
   - Delete oldest files when total size exceeds `max_total_size_mb`
   - Delete oldest files when file count exceeds `max_files`
5. **Configuration**: All settings live in `config.toml` under a new `[logging]` section.

---

## Files Changed

| File | Change Type | Description |
|------|-------------|-------------|
| `Cargo.toml` | Modify | Add `tracing-appender = "0.2"` |
| `config.toml` | Modify | Add `[logging]` section |
| `src/config.rs` | Modify | Add `LoggingConfig` struct + defaults + validation |
| `src/logging.rs` | **New** | `init_logging()` + `cleanup_old_logs()` |
| `src/main.rs` | Modify | Wire `mod logging`, replace init, spawn cleanup task |
| `.gitignore` | Modify | Add `logs/` |

---

## Task 1: Add Dependency in `Cargo.toml`

Under the `# Logging` section, add:

```toml
tracing-appender = "0.2"
```

The existing dependencies remain unchanged:

```toml
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

---

## Task 2: Add `[logging]` Section in `config.toml`

Append to the end of `config.toml`:

```toml
[logging]
dir = "./logs"               # Log output directory
level = "info"               # Log level: trace, debug, info, warn, error
max_files = 30               # Max number of log files to retain
max_days = 30                # Max days to keep log files (based on file mtime)
max_total_size_mb = 500      # Max total size of all log files in MB
console_output = true        # Also print logs to console (stdout)
```

---

## Task 3: Add `LoggingConfig` in `src/config.rs`

### 3a. Define default functions

```rust
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
```

### 3b. Define `LoggingConfig` struct

```rust
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
```

### 3c. Add field to `AppConfig`

```rust
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
```

Note: `#[serde(default)]` ensures the app still starts if `[logging]` is missing from config.toml.

### 3d. Add validation in `AppConfig::validate()`

```rust
// Logging validation
if self.logging.dir.is_empty() {
    return Err("logging.dir must not be empty".into());
}
let valid_levels = ["trace", "debug", "info", "warn", "error"];
if !valid_levels.contains(&self.logging.level.to_lowercase().as_str()) {
    return Err(format!("logging.level must be one of: {:?}", valid_levels).into());
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
```

---

## Task 4: Create `src/logging.rs` — Log Initialization + Cleanup

### 4a. Module structure

```rust
use crate::config::{AppConfig, LoggingConfig};
use std::path::Path;
use std::time::Duration;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize dual-output logging (file + optional console).
pub fn init_logging(config: &AppConfig) {
    let log_cfg = &config.logging;

    // 1. Ensure log directory exists
    std::fs::create_dir_all(&log_cfg.dir).expect("Failed to create log directory");

    // 2. Build rolling file appender (daily rotation)
    let file_appender = RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .max_log_files(log_cfg.max_files as usize)
        .build(&log_cfg.dir)
        .expect("Failed to create rolling file appender");

    // 3. Build env filter from config level
    let env_filter = EnvFilter::try_new(&log_cfg.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 4. File layer (always enabled)
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false);

    // 5. Compose subscriber
    if log_cfg.console_output {
        let console_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }
}
```

### 4b. Log cleanup function

```rust
/// Remove old log files based on max_days, max_total_size_mb, and max_files.
pub fn cleanup_old_logs(config: &LoggingConfig) {
    let log_dir = Path::new(&config.dir);
    if !log_dir.exists() {
        return;
    }

    let max_age = Duration::from_secs(config.max_days as u64 * 86400);
    let max_total_bytes = config.max_total_size_mb as u64 * 1024 * 1024;
    let now = std::time::SystemTime::now();

    // Collect all .log files with metadata
    let mut log_files: Vec<(std::path::PathBuf, std::time::SystemTime, u64)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("log") {
                if let Ok(meta) = path.metadata() {
                    let mtime = meta.modified().unwrap_or(now);
                    log_files.push((path, mtime, meta.len()));
                }
            }
        }
    }

    // Sort oldest first
    log_files.sort_by_key(|(_, mtime, _)| *mtime);

    let mut removed = 0usize;

    // --- Rule 1: Remove files older than max_days ---
    log_files.retain(|(path, mtime, _size)| {
        if let Ok(age) = now.duration_since(*mtime) {
            if age > max_age {
                tracing::debug!("Cleanup: removing old log file {:?}", path);
                let _ = std::fs::remove_file(path);
                removed += 1;
                return false;
            }
        }
        true
    });

    // --- Rule 2: Remove oldest files if total size exceeds limit ---
    let total_size: u64 = log_files.iter().map(|(_, _, s)| *s).sum();
    if total_size > max_total_bytes {
        let mut excess = total_size - max_total_bytes;
        log_files.retain(|(path, _, size)| {
            if excess > 0 {
                tracing::debug!("Cleanup: removing oversized log file {:?}", path);
                let _ = std::fs::remove_file(path);
                excess = excess.saturating_sub(*size);
                removed += 1;
                return false;
            }
            true
        });
    }

    // --- Rule 3: Remove oldest files if count exceeds max_files ---
    while log_files.len() > config.max_files as usize {
        if let Some((path, _, _)) = log_files.first() {
            tracing::debug!("Cleanup: removing excess log file {:?}", path);
            let _ = std::fs::remove_file(path);
            removed += 1;
        }
        log_files.remove(0);
    }

    if removed > 0 {
        tracing::info!("Log cleanup complete: removed {} old log file(s)", removed);
    }
}
```

---

## Task 5: Wire into `src/main.rs`

### 5a. Add module declaration

At the top of `main.rs`, add:

```rust
mod logging;
```

### 5b. Replace logging initialization

**Before** (line 95):

```rust
tracing_subscriber::fmt().with_env_filter("info").init();
```

**After**:

```rust
// Config path from first CLI argument, default to "config.toml"
let config_path = std::env::args()
    .nth(1)
    .unwrap_or_else(|| "config.toml".to_string());
let config = config::AppConfig::load(&config_path)?;

// Initialize file + console logging
logging::init_logging(&config);

// Run initial log cleanup
logging::cleanup_old_logs(&config.logging);
```

Note: Move the config loading **before** logging init so that log settings are available. The current order already loads config after the old tracing init — we just swap positions.

### 5c. Spawn periodic cleanup task

After spawning the Ctrl+C listener and before spawning the pipeline tasks:

```rust
// Periodic log cleanup (every 6 hours)
let log_config = config.logging.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 3600));
    loop {
        interval.tick().await;
        logging::cleanup_old_logs(&log_config);
    }
});
```

### 5d. Updated `main()` flow (conceptual)

```
1. Load config
2. logging::init_logging(&config)        // replaces old tracing init
3. logging::cleanup_old_logs(...)        // one-shot cleanup on startup
4. Create DB dir + pool + migrations
5. ensure_initial_token
6. Spawn periodic log cleanup task       // every 6h
7. Spawn parser / filter / pusher
8. Start axum server
9. Graceful shutdown
```

---

## Task 6: Add `logs/` to `.gitignore`

Append to `.gitignore`:

```
# Log output
logs/
```

---

## Summary

After implementation:

- All `tracing::error!`, `tracing::warn!`, `tracing::info!` calls are written to both
  `./logs/app-YYYY-MM-DD.log` files and optionally stdout.
- Log files rotate daily.
- On startup and every 6 hours, old/excess log files are cleaned up based on:
  - `max_days` (age-based removal)
  - `max_total_size_mb` (size-based removal)
  - `max_files` (count-based removal)
- All behavior is configurable via `config.toml` without recompilation.
- `./logs/` is excluded from version control.
