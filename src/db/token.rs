use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::models::token::ApiToken;

pub async fn create_token(
    pool: &SqlitePool,
    name: &str,
    token: &str,
    expires_at: Option<NaiveDateTime>,
) -> Result<ApiToken, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        "INSERT INTO api_tokens (name, token, expires_at) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(name)
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn list_tokens(pool: &SqlitePool) -> Result<Vec<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        "SELECT * FROM api_tokens WHERE revoked = 0 ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_token_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_token_by_value(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE token = ? AND revoked = 0")
        .bind(token)
        .fetch_optional(pool)
        .await
}

pub async fn update_token_last_used(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE api_tokens SET last_used_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_token(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE api_tokens SET revoked = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_token(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM api_tokens WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
