use chrono::NaiveDateTime;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::models::push_record::PushRecord;

/// Atomically claim pending push_records by updating status to 'processing'.
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
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(status)
    .bind(retry_count)
    .bind(next_retry_at)
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
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = ?",
    )
    .bind(new_status)
    .bind(retry_count)
    .bind(next_retry_at)
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
         pr.status, pr.retry_count, pr.next_retry_at, pr.created_at, pr.updated_at \
         FROM push_records pr \
         JOIN push_channels pc ON pc.id = pr.channel_id \
         WHERE pr.hot_event_id = ? \
         ORDER BY pr.channel_id",
    )
    .bind(hot_event_id)
    .fetch_all(pool)
    .await
}
