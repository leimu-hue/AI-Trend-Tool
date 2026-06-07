mod config;
mod db;
mod error;
mod models;
mod routes;

use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "hotspot", about = "AI Trend Monitor")]
struct Cli {
    #[arg(long, default_value = "config.toml")]
    config: String,

    #[arg(default_value = "all")]
    mode: String, // all | api | parser | filter | pusher
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

    // Build router
    let app = routes::create_router(pool.clone(), config.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
