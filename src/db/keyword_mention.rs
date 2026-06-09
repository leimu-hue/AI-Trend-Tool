use sqlx::SqlitePool;

/// Batch insert keyword mentions in chunks of 100 to avoid SQLite variable limits.
/// Each chunk uses a single INSERT with multiple value tuples.
pub async fn batch_insert_keyword_mentions(
    pool: &SqlitePool,
    mentions: &[(i64, i64)],
) -> Result<(), sqlx::Error> {
    for chunk in mentions.chunks(100) {
        let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?)".to_string()).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO keyword_mentions (keyword_id, article_id) VALUES {}",
            placeholders.join(", ")
        );
        let mut query = sqlx::query(&sql);
        for (keyword_id, article_id) in chunk {
            query = query.bind(*keyword_id).bind(*article_id);
        }
        query.execute(pool).await?;
    }
    Ok(())
}
