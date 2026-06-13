use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Valid article status values
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_PROCESSING: &str = "processing";
pub const STATUS_MATCHED: &str = "matched";
pub const STATUS_SKIPPED: &str = "skipped";

/// All valid article statuses
pub const VALID_ARTICLE_STATUSES: &[&str] = &[
    STATUS_PENDING,
    STATUS_PROCESSING,
    STATUS_MATCHED,
    STATUS_SKIPPED,
];

#[derive(Debug, FromRow, Serialize)]
pub struct Article {
    pub id: i64,
    pub source_id: i64,
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub published_at: Option<NaiveDateTime>,
    pub fetched_at: NaiveDateTime,
    pub processed_at: Option<NaiveDateTime>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_id: Option<i64>,
    /// DEPRECATED: use `status` instead. `processed=true` → `status=matched`, `processed=false` → `status=pending`.
    pub processed: Option<bool>,
    /// Filter by article status: pending | processing | matched | skipped
    pub status: Option<String>,
}
