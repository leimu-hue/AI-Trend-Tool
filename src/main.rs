mod config;
mod db;
mod error;
mod handlers;
mod logging;
mod middleware;
mod models;
mod pipeline;
mod routes;
mod services;

use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::config::AppConfig;
use crate::pipeline::Pipeline;

/// Ensure at least one API token exists in the database.
/// On first startup, creates an initial admin token from config or auto-generates one.
/// Always prints the active initial token to console for easy copy-paste.
pub async fn ensure_initial_token(
    pool: &SqlitePool,
    config: &AppConfig,
) -> Result<(), sqlx::Error> {
    let count = db::token::count_all_tokens(pool).await?;

    if count > 0 {
        // Tokens already exist — print masked form of the first non-revoked one
        if let Some(token) = db::token::get_first_active_token(pool).await? {
            let t = &token.token;
            let masked = if t.len() >= 8 {
                format!("{}...{}", &t[..4], &t[t.len() - 4..])
            } else {
                "****".to_string()
            };
            tracing::info!("============================================");
            tracing::info!("  Active token: {}", masked);
            tracing::info!("  ({} token(s) total in database)", count);
            tracing::info!("============================================");
        }
        return Ok(());
    }

    let token_str = match &config.auth.initial_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            use rand::Rng;
            let bytes: [u8; 32] = rand::thread_rng().gen();
            hex::encode(bytes)
        }
    };

    db::token::insert_initial_token(pool, "Initial Admin Token", &token_str).await?;

    tracing::warn!("============================================");
    tracing::warn!("  INITIAL TOKEN (save this!): {}", token_str);
    tracing::warn!("============================================");
    Ok(())
}

/// Run migrations with automatic recovery from checksum mismatches.
///
/// `sqlx::migrate!()` embeds migration file checksums at compile time. If a
/// migration SQL file was edited after the DB already recorded its checksum,
/// sqlx raises `VersionMismatch`. Instead of re-running non-idempotent DDL,
/// we simply **update the stored checksums** to match the current binary.
async fn run_migrations(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let migrator = sqlx::migrate!("./docs/migrations");

    match migrator.run(pool).await {
        Ok(()) => Ok(()),
        Err(sqlx::migrate::MigrateError::VersionMismatch(v)) => {
            tracing::warn!(
                "Migration checksum mismatch at version {} — updating stored checksums to match current binary",
                v
            );
            for migration in migrator.iter() {
                sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
                    .bind(migration.checksum.as_ref())
                    .bind(migration.version)
                    .execute(pool)
                    .await?;
            }
            tracing::info!("Migration checksums updated successfully");
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Config path from first CLI argument, default to "config.toml"
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());
    let config = config::AppConfig::load(&config_path)?;

    // Initialize file + console logging (replaces old tracing_subscriber::fmt init)
    logging::init_logging(&config);

    // Run log cleanup on startup
    logging::cleanup_old_logs(&config.logging);

    // Ensure data directory exists
    let db_dir = std::path::Path::new(&config.database.path)
        .parent()
        .ok_or("database.path has no parent directory — specify an absolute or relative path with at least one parent")?;
    std::fs::create_dir_all(db_dir)?;

    // Initialize database connection pool
    let pool = db::init_pool(&config.database.path, config.parser.max_concurrent_fetches).await?;

    // Run migrations (with auto-recovery for checksum mismatches)
    tracing::info!("Database migration starting...");
    run_migrations(&pool).await?;
    tracing::info!("Database migration complete");

    // Ensure at least one API token exists (first startup bootstrap)
    ensure_initial_token(&pool, &config).await?;

    // Create event-driven pipeline
    let (pipeline, articles_rx, push_rx) = Pipeline::new();

    // Ctrl+C listener — signals graceful shutdown
    let cancel_token = pipeline.cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Ctrl+C received, shutting down gracefully...");
        cancel_token.cancel();
    });

    // Spawn three background tasks with event-driven links
    let parser_handle = tokio::spawn(services::parser::start_parser_loop(
        pool.clone(),
        config.parser.clone(),
        pipeline.clone(),
    ));
    let filter_handle = tokio::spawn(services::filter::start_filter_loop(
        pool.clone(),
        config.filter.clone(),
        pipeline.clone(),
        articles_rx,
    ));
    let pusher_handle = tokio::spawn(services::pusher::start_pusher_loop(
        pool.clone(),
        config.pusher.clone(),
        pipeline.clone(),
        push_rx,
    ));

    tracing::info!("Parser + Filter + Pusher running in background");

    // Periodic log cleanup (every 6 hours)
    let log_cfg = config.logging.clone();
    let log_cancel = pipeline.cancel.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 3600));
        loop {
            tokio::select! {
                _ = log_cancel.cancelled() => break,
                _ = interval.tick() => {
                    logging::cleanup_old_logs(&log_cfg);
                }
            }
        }
    });

    // Build router and start API server with graceful shutdown
    let app = routes::create_router(pool.clone(), config.clone(), pipeline.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(pipeline.cancel.cancelled_owned())
        .await?;

    // Wait for background tasks to finish gracefully
    let _ = tokio::join!(parser_handle, filter_handle, pusher_handle);

    tracing::info!("Shutdown complete");
    Ok(())
}
