use sqlx::SqlitePool;

use crate::models::keyword::{CreateKeywordRequest, Keyword, UpdateKeywordRequest};

pub async fn create_keyword(
    pool: &SqlitePool,
    req: &CreateKeywordRequest,
) -> Result<Keyword, sqlx::Error> {
    sqlx::query_as::<_, Keyword>(
        "INSERT INTO keywords (word, case_sensitive, std_multiplier, min_hot_count) \
         VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(&req.word)
    .bind(req.case_sensitive.unwrap_or(false))
    .bind(req.std_multiplier.unwrap_or(2.0))
    .bind(req.min_hot_count.unwrap_or(3))
    .fetch_one(pool)
    .await
}

pub async fn list_keywords(pool: &SqlitePool) -> Result<Vec<Keyword>, sqlx::Error> {
    sqlx::query_as::<_, Keyword>("SELECT * FROM keywords ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn list_enabled_keywords(pool: &SqlitePool) -> Result<Vec<Keyword>, sqlx::Error> {
    sqlx::query_as::<_, Keyword>("SELECT * FROM keywords WHERE enabled = 1 ORDER BY word")
        .fetch_all(pool)
        .await
}

pub async fn get_keyword_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Keyword>, sqlx::Error> {
    sqlx::query_as::<_, Keyword>("SELECT * FROM keywords WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_keyword(
    pool: &SqlitePool,
    id: i64,
    req: &UpdateKeywordRequest,
) -> Result<Option<Keyword>, sqlx::Error> {
    let mut sets: Vec<&str> = vec![];
    if req.word.is_some() {
        sets.push("word = ?");
    }
    if req.case_sensitive.is_some() {
        sets.push("case_sensitive = ?");
    }
    if req.enabled.is_some() {
        sets.push("enabled = ?");
    }
    if req.std_multiplier.is_some() {
        sets.push("std_multiplier = ?");
    }
    if req.min_hot_count.is_some() {
        sets.push("min_hot_count = ?");
    }
    if sets.is_empty() {
        return get_keyword_by_id(pool, id).await;
    }

    let sql = format!(
        "UPDATE keywords SET {} WHERE id = ? RETURNING *",
        sets.join(", ")
    );

    let mut query = sqlx::query_as::<_, Keyword>(&sql);
    if let Some(ref v) = req.word {
        query = query.bind(v);
    }
    if let Some(v) = req.case_sensitive {
        query = query.bind(v);
    }
    if let Some(v) = req.enabled {
        query = query.bind(v);
    }
    if let Some(v) = req.std_multiplier {
        query = query.bind(v);
    }
    if let Some(v) = req.min_hot_count {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(pool).await
}

pub async fn delete_keyword(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM keywords WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
