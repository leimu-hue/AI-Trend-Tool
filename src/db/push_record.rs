use chrono::NaiveDateTime;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::models::push_record::PushRecord;

/// Atomically claim pending push_records by updating status to 'processing'.
/// Excludes 'dead' records from claiming.
/// Returns number of claimed rows.
pub async fn claim_pending_records(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records SET status = 'processing', updated_at = datetime('now') \
         WHERE status = 'pending' \
         AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Recover push_records stuck in 'processing' for longer than `timeout_minutes`.
/// Returns the number of recovered records.
pub async fn recover_stale_processing_records(
    pool: &SqlitePool,
    timeout_minutes: u64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records SET status = 'pending', updated_at = datetime('now') \
         WHERE status = 'processing' \
         AND updated_at < datetime('now', ? || ' minutes')",
    )
    .bind(format!("-{}", timeout_minutes))
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// List push_records with status = 'processing' (claimed for delivery).
pub async fn list_processing_records(pool: &SqlitePool) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records WHERE status = 'processing' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

/// Insert push records for all enabled channels for a given hot event.
/// Skips channels that already have a record (UNIQUE constraint).
/// Logs real database errors via tracing::error.
#[allow(dead_code)]
pub async fn insert_push_records_for_event(
    pool: &SqlitePool,
    hot_event_id: i64,
    channel_ids: &[i64],
) -> Result<Vec<PushRecord>, sqlx::Error> {
    let mut records = vec![];
    for &channel_id in channel_ids {
        match sqlx::query_as::<_, PushRecord>(
            "INSERT OR IGNORE INTO push_records (hot_event_id, channel_id) VALUES (?, ?) RETURNING *",
        )
        .bind(hot_event_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(r)) => records.push(r),
            Ok(None) => { /* UNIQUE conflict — normal dedup, skip */ }
            Err(e) => {
                tracing::error!(
                    "Failed to insert push_record for hot_event {} channel {}: {}",
                    hot_event_id,
                    channel_id,
                    e
                );
            }
        }
    }
    Ok(records)
}

/// Transaction-aware version of insert_push_records_for_event.
/// Logs real database errors via tracing::error.
pub async fn insert_push_records_for_event_tx(
    tx: &mut Transaction<'_, Sqlite>,
    hot_event_id: i64,
    channel_ids: &[i64],
) -> Result<Vec<PushRecord>, sqlx::Error> {
    let mut records = vec![];
    for &channel_id in channel_ids {
        match sqlx::query_as::<_, PushRecord>(
            "INSERT OR IGNORE INTO push_records (hot_event_id, channel_id) VALUES (?, ?) RETURNING *",
        )
        .bind(hot_event_id)
        .bind(channel_id)
        .fetch_optional(&mut **tx)
        .await
        {
            Ok(Some(r)) => records.push(r),
            Ok(None) => { /* UNIQUE conflict — normal dedup, skip */ }
            Err(e) => {
                tracing::error!(
                    "Failed to insert push_record for hot_event {} channel {} in tx: {}",
                    hot_event_id,
                    channel_id,
                    e
                );
            }
        }
    }
    Ok(records)
}

#[allow(dead_code)]
pub async fn list_pending_records(pool: &SqlitePool) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records WHERE status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_retry_due_records(
    pool: &SqlitePool,
    max_retries: u32,
) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records \
         WHERE status = 'failed' \
         AND retry_count < ? \
         AND next_retry_at <= datetime('now') \
         ORDER BY next_retry_at ASC",
    )
    .bind(max_retries as i32)
    .fetch_all(pool)
    .await
}

pub async fn update_push_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    retry_count: i32,
    next_retry_at: Option<NaiveDateTime>,
    last_error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, last_error = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(status)
    .bind(retry_count)
    .bind(next_retry_at)
    .bind(last_error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Optimistic lock: only update if current status matches expected
pub async fn update_push_status_optimistic(
    pool: &SqlitePool,
    id: i64,
    expected_status: &str,
    new_status: &str,
    retry_count: i32,
    next_retry_at: Option<NaiveDateTime>,
    last_error: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, last_error = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = ?",
    )
    .bind(new_status)
    .bind(retry_count)
    .bind(next_retry_at)
    .bind(last_error)
    .bind(id)
    .bind(expected_status)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Push record with channel name (for API response)
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PushRecordWithChannel {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub channel_name: String,
    pub status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<chrono::NaiveDateTime>,
    pub last_error: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// List push records with channel names for a hot event
pub async fn list_push_records_with_details(
    pool: &SqlitePool,
    hot_event_id: i64,
) -> Result<Vec<PushRecordWithChannel>, sqlx::Error> {
    sqlx::query_as::<_, PushRecordWithChannel>(
        "SELECT pr.id, pr.hot_event_id, pr.channel_id, pc.name as channel_name, \
         pr.status, pr.retry_count, pr.next_retry_at, pr.last_error, pr.created_at, pr.updated_at \
         FROM push_records pr \
         JOIN push_channels pc ON pc.id = pr.channel_id \
         WHERE pr.hot_event_id = ? \
         ORDER BY pr.channel_id",
    )
    .bind(hot_event_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./docs/migrations")
            .run(&pool)
            .await
            .unwrap();
        pool
    }

    /// Insert test data: keyword, data_source, article, hot_event, push_channel, push_record
    async fn seed_push_test_data(pool: &SqlitePool) -> (i64, i64, i64) {
        // Insert keyword
        let kw = sqlx::query_as::<_, crate::models::keyword::Keyword>(
            "INSERT INTO keywords (word, enabled) VALUES ('test-kw', 1) RETURNING *",
        )
        .fetch_one(pool)
        .await
        .unwrap();

        // Insert data source
        sqlx::query(
            "INSERT INTO data_sources (type, name, url) VALUES ('rss', 'Test', 'http://x')",
        )
        .execute(pool)
        .await
        .unwrap();

        // Insert article
        let article = sqlx::query_as::<_, crate::models::article::Article>(
            "INSERT INTO articles (source_id, link, title, status) VALUES (1, 'http://test', 'Test', 'matched') RETURNING *",
        )
        .fetch_one(pool)
        .await
        .unwrap();

        // Insert hot_event
        let he = sqlx::query_as::<_, crate::models::hot_event::HotEvent>(
            "INSERT INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical) \
             VALUES (?, '2026061300', 10, 5.0, 2.0) RETURNING *",
        )
        .bind(kw.id)
        .fetch_one(pool)
        .await
        .unwrap();

        // Insert push_channel
        sqlx::query(
            "INSERT INTO push_channels (name, channel_type, config, enabled) \
             VALUES ('Test Channel', 'webhook', '{\"url\":\"http://hook\"}', 1)",
        )
        .execute(pool)
        .await
        .unwrap();

        (kw.id, article.id, he.id)
    }

    // ── 9.5: recover_stale_processing_records tests ──

    #[tokio::test]
    async fn recover_stale_processing_no_stale_records() {
        let pool = setup_test_db().await;
        let (_, _, he_id) = seed_push_test_data(&pool).await;

        // Insert a push_record with status=processing, just updated
        sqlx::query(
            "INSERT INTO push_records (hot_event_id, channel_id, status) VALUES (?, 1, 'processing')",
        )
        .bind(he_id)
        .execute(&pool)
        .await
        .unwrap();

        // Recovery with long timeout should find nothing (record is fresh)
        let recovered = recover_stale_processing_records(&pool, 10).await.unwrap();
        assert_eq!(recovered, 0);
    }

    #[tokio::test]
    async fn recover_stale_processing_recovers_stuck_records() {
        let pool = setup_test_db().await;
        let (_, _, he_id) = seed_push_test_data(&pool).await;

        // Insert a push_record with status=processing, but with old updated_at
        // We need to manually set updated_at to a past time
        sqlx::query(
            "INSERT INTO push_records (hot_event_id, channel_id, status, updated_at) \
             VALUES (?, 1, 'processing', datetime('now', '-20 minutes'))",
        )
        .bind(he_id)
        .execute(&pool)
        .await
        .unwrap();

        // Recovery with 10 min timeout should find and recover it
        let recovered = recover_stale_processing_records(&pool, 10).await.unwrap();
        assert_eq!(recovered, 1);

        // Verify the record is now pending
        let records = list_processing_records(&pool).await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn recover_stale_does_not_touch_pending_or_success() {
        let pool = setup_test_db().await;
        let (_, _, he_id) = seed_push_test_data(&pool).await;

        // Insert additional push channels for different (he_id, channel_id) combos
        sqlx::query(
            "INSERT INTO push_channels (name, channel_type, config, enabled) \
             VALUES ('Ch2', 'webhook', '{\"url\":\"http://h2\"}', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO push_channels (name, channel_type, config, enabled) \
             VALUES ('Ch3', 'webhook', '{\"url\":\"http://h3\"}', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert records with different statuses and channel_ids, all old
        sqlx::query(
            "INSERT INTO push_records (hot_event_id, channel_id, status, updated_at) \
             VALUES (?, 2, 'pending', datetime('now', '-20 minutes'))",
        )
        .bind(he_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO push_records (hot_event_id, channel_id, status, updated_at) \
             VALUES (?, 3, 'success', datetime('now', '-20 minutes'))",
        )
        .bind(he_id)
        .execute(&pool)
        .await
        .unwrap();

        // Recovery should only touch processing records
        let recovered = recover_stale_processing_records(&pool, 10).await.unwrap();
        assert_eq!(recovered, 0);
    }
}
