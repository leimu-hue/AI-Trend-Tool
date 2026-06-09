use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::hot_event::HotEvent;

pub async fn get_hot_event_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<HotEvent>, sqlx::Error> {
    sqlx::query_as::<_, HotEvent>("SELECT * FROM hot_events WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Paginated hotspot listing with optional keyword_id filter
pub async fn list_hotspots_paginated(
    pool: &SqlitePool,
    keyword_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<HotEvent>, sqlx::Error> {
    if let Some(kid) = keyword_id {
        sqlx::query_as::<_, HotEvent>(
            "SELECT * FROM hot_events WHERE keyword_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(kid)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, HotEvent>(
            "SELECT * FROM hot_events ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Count hotspots with optional keyword_id filter
pub async fn count_hotspots(
    pool: &SqlitePool,
    keyword_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = if let Some(kid) = keyword_id {
        sqlx::query_as("SELECT COUNT(*) as count FROM hot_events WHERE keyword_id = ?")
            .bind(kid)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as("SELECT COUNT(*) as count FROM hot_events")
            .fetch_one(pool)
            .await?
    };
    Ok(row.0)
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

/// Get hourly counts for all keywords in the last N hours as a single batch query.
/// Returns (keyword_id, hour_bucket, count) tuples.
pub async fn get_all_hourly_counts(
    pool: &SqlitePool,
    hours: i32,
) -> Result<Vec<(i64, String, i32)>, sqlx::Error> {
    let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
    let cutoff_str = cutoff.format("%Y%m%d%H").to_string();
    sqlx::query_as::<_, (i64, String, i32)>(
        "SELECT keyword_id, hour_bucket, SUM(count) as total \
         FROM hot_events \
         WHERE hour_bucket >= ? \
         GROUP BY keyword_id, hour_bucket \
         ORDER BY hour_bucket",
    )
    .bind(cutoff_str)
    .fetch_all(pool)
    .await
}
