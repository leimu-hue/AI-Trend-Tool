use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct HotEvent {
    pub id: i64,
    pub keyword_id: i64,
    pub hour_bucket: String,
    pub count: i32,
    pub mean_historical: f64,
    pub stddev_historical: f64,
    pub created_at: NaiveDateTime,
}
