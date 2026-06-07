use sqlx::SqlitePool;

use crate::models::channel::{CreateChannelRequest, PushChannel, UpdateChannelRequest};

pub async fn create_channel(
    pool: &SqlitePool,
    req: &CreateChannelRequest,
) -> Result<PushChannel, sqlx::Error> {
    let channel_type = req.channel_type.as_deref().unwrap_or("webhook");
    sqlx::query_as::<_, PushChannel>(
        "INSERT INTO push_channels (name, channel_type, config) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&req.name)
    .bind(channel_type)
    .bind(&req.config)
    .fetch_one(pool)
    .await
}

pub async fn list_channels(pool: &SqlitePool) -> Result<Vec<PushChannel>, sqlx::Error> {
    sqlx::query_as::<_, PushChannel>("SELECT * FROM push_channels ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn list_enabled_channels(pool: &SqlitePool) -> Result<Vec<PushChannel>, sqlx::Error> {
    sqlx::query_as::<_, PushChannel>("SELECT * FROM push_channels WHERE enabled = 1")
        .fetch_all(pool)
        .await
}

pub async fn get_channel_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<PushChannel>, sqlx::Error> {
    sqlx::query_as::<_, PushChannel>("SELECT * FROM push_channels WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_channel(
    pool: &SqlitePool,
    id: i64,
    req: &UpdateChannelRequest,
) -> Result<Option<PushChannel>, sqlx::Error> {
    let mut sets: Vec<&str> = vec![];
    if req.name.is_some() {
        sets.push("name = ?");
    }
    if req.config.is_some() {
        sets.push("config = ?");
    }
    if req.enabled.is_some() {
        sets.push("enabled = ?");
    }
    if sets.is_empty() {
        return get_channel_by_id(pool, id).await;
    }

    let sql = format!(
        "UPDATE push_channels SET {} WHERE id = ? RETURNING *",
        sets.join(", ")
    );

    let mut query = sqlx::query_as::<_, PushChannel>(&sql);
    if let Some(ref v) = req.name {
        query = query.bind(v);
    }
    if let Some(ref v) = req.config {
        query = query.bind(v);
    }
    if let Some(v) = req.enabled {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(pool).await
}

pub async fn delete_channel(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM push_channels WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
