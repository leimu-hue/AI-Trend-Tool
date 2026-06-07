use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_id: Option<i64>,
    pub processed: Option<bool>,
}
