use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Keyword {
    pub id: i64,
    pub word: String,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub std_multiplier: f64,
    pub min_hot_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeywordRequest {
    pub word: String,
    pub case_sensitive: Option<bool>,
    pub std_multiplier: Option<f64>,
    pub min_hot_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeywordRequest {
    pub word: Option<String>,
    pub case_sensitive: Option<bool>,
    pub enabled: Option<bool>,
    pub std_multiplier: Option<f64>,
    pub min_hot_count: Option<i32>,
}
