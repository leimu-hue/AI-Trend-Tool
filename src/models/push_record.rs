use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct PushRecord {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
