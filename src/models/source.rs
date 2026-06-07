use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DataSource {
    pub id: i64,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub config: String, // JSON string
    pub enabled: bool,
    pub interval_seconds: i64,
    pub last_fetched_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateSourceRequest {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub interval_seconds: Option<i64>,
    pub config: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<bool>,
    pub interval_seconds: Option<i64>,
    pub config: Option<String>,
}
