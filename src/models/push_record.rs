use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

/// Valid push record status values
#[allow(dead_code)]
pub const PR_STATUS_PENDING: &str = "pending";
#[allow(dead_code)]
pub const PR_STATUS_PROCESSING: &str = "processing";
#[allow(dead_code)]
pub const PR_STATUS_SUCCESS: &str = "success";
#[allow(dead_code)]
pub const PR_STATUS_FAILED: &str = "failed";
#[allow(dead_code)]
pub const PR_STATUS_DEAD: &str = "dead";

#[derive(Debug, FromRow, Serialize)]
pub struct PushRecord {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
