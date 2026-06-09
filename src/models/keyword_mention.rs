use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
#[allow(dead_code)]
pub struct KeywordMention {
    pub id: i64,
    pub keyword_id: i64,
    pub article_id: i64,
    pub matched_at: NaiveDateTime,
}
