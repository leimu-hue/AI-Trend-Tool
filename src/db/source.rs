use sqlx::SqlitePool;

use crate::models::source::{CreateSourceRequest, DataSource, UpdateSourceRequest};

pub async fn create_source(
    pool: &SqlitePool,
    req: &CreateSourceRequest,
) -> Result<DataSource, sqlx::Error> {
    let interval = req.interval_seconds.unwrap_or(300);
    let config = req.config.as_deref().unwrap_or("{}");
    sqlx::query_as::<_, DataSource>(
        "INSERT INTO data_sources (type, name, url, interval_seconds, config) \
         VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&req.source_type)
    .bind(&req.name)
    .bind(&req.url)
    .bind(interval)
    .bind(config)
    .fetch_one(pool)
    .await
}

pub async fn list_sources(pool: &SqlitePool) -> Result<Vec<DataSource>, sqlx::Error> {
    sqlx::query_as::<_, DataSource>(
        "SELECT * FROM data_sources ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_source_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<DataSource>, sqlx::Error> {
    sqlx::query_as::<_, DataSource>("SELECT * FROM data_sources WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_source(
    pool: &SqlitePool,
    id: i64,
    req: &UpdateSourceRequest,
) -> Result<Option<DataSource>, sqlx::Error> {
    // Build dynamic UPDATE query for partial updates
    let mut sets: Vec<String> = vec![];
    if req.name.is_some() {
        sets.push("name = ?".into());
    }
    if req.url.is_some() {
        sets.push("url = ?".into());
    }
    if req.enabled.is_some() {
        sets.push("enabled = ?".into());
    }
    if req.interval_seconds.is_some() {
        sets.push("interval_seconds = ?".into());
    }
    if req.config.is_some() {
        sets.push("config = ?".into());
    }
    if sets.is_empty() {
        return get_source_by_id(pool, id).await;
    }
    sets.push("updated_at = datetime('now')".into());

    let sql = format!(
        "UPDATE data_sources SET {} WHERE id = ? RETURNING *",
        sets.join(", ")
    );

    let mut query = sqlx::query_as::<_, DataSource>(&sql);
    if let Some(ref v) = req.name {
        query = query.bind(v);
    }
    if let Some(ref v) = req.url {
        query = query.bind(v);
    }
    if let Some(v) = req.enabled {
        query = query.bind(v);
    }
    if let Some(v) = req.interval_seconds {
        query = query.bind(v);
    }
    if let Some(ref v) = req.config {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(pool).await
}

pub async fn delete_source(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM data_sources WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_source_last_fetched(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE data_sources SET last_fetched_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
