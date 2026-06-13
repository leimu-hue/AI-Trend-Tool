use serde::Serialize;
use sqlx::SqlitePool;

use crate::models::source::{CreateSourceRequest, DataSource, UpdateSourceRequest};

/// DataSource with article count from LEFT JOIN
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct SourceWithCount {
    pub id: i64,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub config: String,
    pub enabled: bool,
    pub interval_seconds: i64,
    pub last_fetched_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub article_count: i64,
}

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

#[allow(dead_code)]
pub async fn list_sources(pool: &SqlitePool) -> Result<Vec<DataSource>, sqlx::Error> {
    sqlx::query_as::<_, DataSource>("SELECT * FROM data_sources ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn list_sources_with_count(
    pool: &SqlitePool,
) -> Result<Vec<SourceWithCount>, sqlx::Error> {
    sqlx::query_as::<_, SourceWithCount>(
        "SELECT ds.id, ds.type, ds.name, ds.url, ds.config, ds.enabled, \
         ds.interval_seconds, ds.last_fetched_at, ds.created_at, ds.updated_at, \
         COUNT(a.id) as article_count \
         FROM data_sources ds \
         LEFT JOIN articles a ON a.source_id = ds.id \
         GROUP BY ds.id \
         ORDER BY ds.created_at DESC",
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

pub async fn update_source_last_fetched(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE data_sources SET last_fetched_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Reset last_fetched_at to NULL so the Parser picks up this source
/// on its next polling cycle (used by manual fetch trigger).
pub async fn reset_last_fetched(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE data_sources SET last_fetched_at = NULL WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List enabled sources that are due for fetching.
/// A source is due when `last_fetched_at IS NULL` or the elapsed time
/// since `last_fetched_at` exceeds its `interval_seconds`.
pub async fn list_due_sources(pool: &SqlitePool) -> Result<Vec<DataSource>, sqlx::Error> {
    sqlx::query_as::<_, DataSource>(
        "SELECT * FROM data_sources \
         WHERE enabled = 1 \
         AND (last_fetched_at IS NULL \
              OR (strftime('%s', 'now') - strftime('%s', last_fetched_at)) >= interval_seconds) \
         ORDER BY last_fetched_at ASC NULLS FIRST",
    )
    .fetch_all(pool)
    .await
}
