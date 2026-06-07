use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::models::push_record::PushRecord;

pub async fn insert_push_record(
    pool: &SqlitePool,
    hot_event_id: i64,
    channel_id: i64,
) -> Result<PushRecord, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "INSERT INTO push_records (hot_event_id, channel_id) VALUES (?, ?) RETURNING *",
    )
    .bind(hot_event_id)
    .bind(channel_id)
    .fetch_one(pool)
    .await
}

/// Insert push records for all enabled channels for a given hot event.
/// Skips channels that already have a record (UNIQUE constraint).
pub async fn insert_push_records_for_event(
    pool: &SqlitePool,
    hot_event_id: i64,
    channel_ids: &[i64],
) -> Result<Vec<PushRecord>, sqlx::Error> {
    let mut records = vec![];
    for &channel_id in channel_ids {
        if let Ok(record) = sqlx::query_as::<_, PushRecord>(
            "INSERT OR IGNORE INTO push_records (hot_event_id, channel_id) VALUES (?, ?) RETURNING *",
        )
        .bind(hot_event_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await
        {
            if let Some(r) = record {
                records.push(r);
            }
        }
    }
    Ok(records)
}

pub async fn list_pending_records(pool: &SqlitePool) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records WHERE status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_retry_due_records(pool: &SqlitePool) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records \
         WHERE status = 'failed' \
         AND retry_count < 3 \
         AND next_retry_at <= datetime('now') \
         ORDER BY next_retry_at ASC",
    )
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

pub async fn get_push_records_by_hot_event(
    pool: &SqlitePool,
    hot_event_id: i64,
) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records WHERE hot_event_id = ? ORDER BY channel_id",
    )
    .bind(hot_event_id)
    .fetch_all(pool)
    .await
}
