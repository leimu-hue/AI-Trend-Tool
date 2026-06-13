use crate::config::{AppConfig, LoggingConfig};
use std::fs;
use std::path::Path;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

/// Initialize the tracing subscriber with file + optional console output.
///
/// Creates the log directory if it doesn't exist, builds a daily-rotating
/// `RollingFileAppender`, and combines a file layer (no ANSI) with an
/// optional console layer into a global subscriber.
pub fn init_logging(config: &AppConfig) {
    let log_cfg = &config.logging;

    // Create log directory if it doesn't exist
    let log_dir = Path::new(&log_cfg.dir);
    fs::create_dir_all(log_dir).expect("Failed to create log directory");

    // Build daily-rotating file appender
    let file_appender =
        tracing_appender::rolling::RollingFileAppender::new(Rotation::DAILY, log_dir, "app");

    // Parse log level filter, fall back to "info"
    let filter = EnvFilter::try_new(&log_cfg.level).unwrap_or_else(|_| EnvFilter::new("info"));

    // File layer — no ANSI colors
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(file_appender)
        .with_filter(filter);

    // Build the subscriber
    if log_cfg.console_output {
        // Console + file
        let console_layer = fmt::layer().with_writer(std::io::stdout).with_filter(
            EnvFilter::try_new(&log_cfg.level).unwrap_or_else(|_| EnvFilter::new("info")),
        );

        tracing_subscriber::registry()
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        // File only
        tracing_subscriber::registry().with(file_layer).init();
    }
}

/// Clean up old log files according to retention policy.
///
/// Rules applied in order:
/// 1. Delete files whose mtime is older than `max_days`.
/// 2. Delete oldest files until total size ≤ `max_total_size_mb`.
/// 3. Delete oldest files until file count ≤ `max_files`.
///
/// Only `.log` files are considered. Non-existent directory is a no-op.
pub fn cleanup_old_logs(config: &LoggingConfig) {
    let dir = Path::new(&config.dir);
    if !dir.exists() {
        return;
    }

    // Collect .log files with their metadata
    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(iter) => iter
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("log") {
                    let meta = fs::metadata(&path).ok()?;
                    let modified = meta.modified().ok()?;
                    let len = meta.len();
                    Some((path, modified, len))
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return,
    };

    if entries.is_empty() {
        return;
    }

    // Sort oldest-first by modified time
    entries.sort_by_key(|(_, mtime, _)| *mtime);

    let now = std::time::SystemTime::now();
    let mut deleted = 0u32;

    // Rule 1: delete files older than max_days
    let max_age = std::time::Duration::from_secs(config.max_days as u64 * 86400);
    entries.retain(|(path, mtime, _)| {
        if let Ok(age) = now.duration_since(*mtime) {
            if age > max_age {
                if fs::remove_file(path).is_ok() {
                    tracing::debug!(
                        "Log cleanup: removed old file {:?} (age > {} days)",
                        path,
                        config.max_days
                    );
                    deleted += 1;
                }
                return false;
            }
        }
        true
    });

    // Rule 2: delete oldest until total size ≤ max_total_size_mb
    if config.max_total_size_mb > 0 {
        let max_bytes = config.max_total_size_mb as u64 * 1024 * 1024;
        let total: u64 = entries.iter().map(|(_, _, len)| len).sum();
        if total > max_bytes {
            let mut excess = total;
            let mut i = 0;
            while i < entries.len() && excess > max_bytes {
                let (path, _, len) = &entries[i];
                if fs::remove_file(path).is_ok() {
                    tracing::debug!(
                        "Log cleanup: removed {:?} (size limit: {} MB)",
                        path,
                        config.max_total_size_mb
                    );
                    excess = excess.saturating_sub(*len);
                    deleted += 1;
                }
                i += 1;
            }
            entries.drain(..i);
        }
    }

    // Rule 3: delete oldest until file count ≤ max_files
    if entries.len() > config.max_files as usize {
        let to_remove = entries.len() - config.max_files as usize;
        for (path, _, _) in entries.iter().take(to_remove) {
            if fs::remove_file(path).is_ok() {
                tracing::debug!(
                    "Log cleanup: removed {:?} (count limit: {} files)",
                    path,
                    config.max_files
                );
                deleted += 1;
            }
        }
    }

    if deleted > 0 {
        tracing::info!("Log cleanup: {} old log file(s) removed", deleted);
    }
}
