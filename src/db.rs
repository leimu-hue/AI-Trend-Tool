pub mod article;
pub mod channel;
pub mod hot_event;
pub mod keyword;
pub mod keyword_mention;
pub mod push_record;
pub mod source;
pub mod token;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn init_pool(
    database_path: &str,
    max_concurrent_fetches: usize,
) -> Result<SqlitePool, sqlx::Error> {
    let max_connections = (max_concurrent_fetches + 5).max(15) as u32;
    let db_url = format!("sqlite:{}?mode=rwc", database_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(&db_url)
        .await?;

    // Enable WAL mode and foreign key enforcement
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    Ok(pool)
}
