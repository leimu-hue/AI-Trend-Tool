use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct PushChannel {
    pub id: i64,
    pub name: String,
    pub channel_type: String,
    pub config: String, // JSON string
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: Option<String>,
    pub config: String, // JSON string: {"url": "https://..."}
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub config: Option<String>,
    pub enabled: Option<bool>,
}
