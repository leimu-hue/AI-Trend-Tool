use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::models::article::{Article, ArticleQuery};

/// Returns Some(Article) if inserted, None if link already exists (deduplicated)
pub async fn insert_article(
    pool: &SqlitePool,
    source_id: i64,
    link: &str,
    title: &str,
    summary: &str,
    content: &str,
    published_at: Option<NaiveDateTime>,
) -> Result<Option<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "INSERT INTO articles (source_id, link, title, summary, content, published_at) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(link) DO NOTHING RETURNING *",
    )
    .bind(source_id)
    .bind(link)
    .bind(title)
    .bind(summary)
    .bind(content)
    .bind(published_at)
    .fetch_optional(pool)
    .await
}

pub async fn list_articles(
    pool: &SqlitePool,
    query: &ArticleQuery,
) -> Result<Vec<Article>, sqlx::Error> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (where_clause, filter_params) = build_article_filter(query);
    let sql = format!(
        "SELECT * FROM articles{} ORDER BY fetched_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut q = sqlx::query_as::<_, Article>(&sql);
    for p in &filter_params {
        q = q.bind(*p);
    }
    q.bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
}

fn build_article_filter(query: &ArticleQuery) -> (String, Vec<i64>) {
    let mut conditions = vec![];
    let mut params: Vec<i64> = vec![];
    if let Some(source_id) = query.source_id {
        conditions.push("source_id = ?".to_string());
        params.push(source_id);
    }
    if let Some(processed) = query.processed {
        if processed {
            conditions.push("processed_at IS NOT NULL".to_string());
        } else {
            conditions.push("processed_at IS NULL".to_string());
        }
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    (where_clause, params)
}

pub async fn get_unprocessed_articles(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT * FROM articles WHERE processed_at IS NULL ORDER BY fetched_at ASC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Bulk update processed_at for a batch of article IDs.
/// Uses IN clause with chunked execution (100 per query) to avoid SQLite variable limits.
pub async fn mark_processed_batch(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for chunk in ids.chunks(100) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE articles SET processed_at = datetime('now') WHERE id IN ({})",
            placeholders
        );
        let mut query = sqlx::query(&sql);
        for id in chunk {
            query = query.bind(*id);
        }
        query.execute(pool).await?;
    }
    Ok(())
}

pub async fn count_articles(pool: &SqlitePool, query: &ArticleQuery) -> Result<i64, sqlx::Error> {
    let (where_clause, filter_params) = build_article_filter(query);
    let sql = format!("SELECT COUNT(*) as count FROM articles{}", where_clause);

    let mut q = sqlx::query_as(&sql);
    for p in &filter_params {
        q = q.bind(*p);
    }
    let count: (i64,) = q.fetch_one(pool).await?;
    Ok(count.0)
}
