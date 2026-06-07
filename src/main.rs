mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;

use clap::Parser;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::config::AppConfig;

#[derive(Parser)]
#[command(name = "hotspot", about = "AI Trend Monitor")]
struct Cli {
    #[arg(long, default_value = "config.toml")]
    config: String,

    #[arg(default_value = "all")]
    mode: String, // all | api | parser | filter | pusher
}

/// Ensure at least one API token exists in the database.
/// On first startup, creates an initial admin token from config or auto-generates one.
/// Always prints the active initial token to console for easy copy-paste.
pub async fn ensure_initial_token(
    pool: &SqlitePool,
    config: &AppConfig,
) -> Result<(), sqlx::Error> {
    let count = db::token::count_all_tokens(pool).await?;

    if count > 0 {
        // Tokens already exist — print the first non-revoked one for convenience
        if let Some(token) = db::token::get_first_active_token(pool).await? {
            tracing::info!("============================================");
            tracing::info!("  Active token: {}", token.token);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let cli = Cli::parse();
    let config = config::AppConfig::load(&cli.config)?;

    // Ensure data directory exists
    let db_dir = std::path::Path::new(&config.database.path)
        .parent()
        .unwrap();
    std::fs::create_dir_all(db_dir)?;

    // Initialize database connection pool
    let pool = db::init_pool(&config.database.path).await?;

    // Run migrations
    sqlx::migrate!("./docs/migrations").run(&pool).await?;

    // Ensure at least one API token exists (first startup bootstrap)
    ensure_initial_token(&pool, &config).await?;

    // Build router
    let app = routes::create_router(pool.clone(), config.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
