use sqlx::SqlitePool;

/// Insert a keyword mention record (keyword_id, article_id).
/// Returns Ok(()) on success; silently ignores duplicates.
pub async fn insert_keyword_mention(
    pool: &SqlitePool,
    keyword_id: i64,
    article_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO keyword_mentions (keyword_id, article_id) VALUES (?, ?)")
        .bind(keyword_id)
        .bind(article_id)
        .execute(pool)
        .await?;
    Ok(())
}
