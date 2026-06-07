use sqlx::SqlitePool;

use crate::models::hot_event::HotEvent;

pub async fn insert_hot_event(
    pool: &SqlitePool,
    keyword_id: i64,
    hour_bucket: &str,
    count: i32,
    mean_historical: f64,
    stddev_historical: f64,
) -> Result<HotEvent, sqlx::Error> {
    sqlx::query_as::<_, HotEvent>(
        "INSERT INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical) \
         VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(keyword_id)
    .bind(hour_bucket)
    .bind(count)
    .bind(mean_historical)
    .bind(stddev_historical)
    .fetch_one(pool)
    .await
}

pub async fn list_hot_events_by_keyword(
    pool: &SqlitePool,
    keyword_id: i64,
    limit: i64,
) -> Result<Vec<HotEvent>, sqlx::Error> {
    sqlx::query_as::<_, HotEvent>(
        "SELECT * FROM hot_events WHERE keyword_id = ? ORDER BY created_at DESC LIMIT ?",
    )
    .bind(keyword_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_hot_events(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<HotEvent>, sqlx::Error> {
    sqlx::query_as::<_, HotEvent>(
        "SELECT * FROM hot_events ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_hot_event_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<HotEvent>, sqlx::Error> {
    sqlx::query_as::<_, HotEvent>("SELECT * FROM hot_events WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get hourly counts for a keyword over recent N hours (for burst detection)
pub async fn get_hourly_counts(
    pool: &SqlitePool,
    keyword_id: i64,
    hours: i32,
) -> Result<Vec<(String, i32)>, sqlx::Error> {
    sqlx::query_as::<_, (String, i32)>(
        "SELECT hour_bucket, SUM(count) as total \
         FROM hot_events \
         WHERE keyword_id = ? \
         GROUP BY hour_bucket \
         ORDER BY hour_bucket DESC \
         LIMIT ?",
    )
    .bind(keyword_id)
    .bind(hours)
    .fetch_all(pool)
    .await
}
