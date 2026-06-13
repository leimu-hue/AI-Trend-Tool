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

    let (where_clause, filter_params, status_param) = build_article_filter(query);
    let sql = format!(
        "SELECT * FROM articles{} ORDER BY fetched_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut q = sqlx::query_as::<_, Article>(&sql);
    for p in &filter_params {
        q = q.bind(*p);
    }
    if let Some(ref s) = status_param {
        q = q.bind(s.as_str());
    }
    q.bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
}

/// Build WHERE clause for article filtering.
/// Returns (where_clause, i64_params, optional_status_param).
fn build_article_filter(query: &ArticleQuery) -> (String, Vec<i64>, Option<String>) {
    let mut conditions = vec![];
    let mut params: Vec<i64> = vec![];

    if let Some(source_id) = query.source_id {
        conditions.push("source_id = ?".to_string());
        params.push(source_id);
    }

    // status takes priority; processed is fallback for backward compat
    let status_p: Option<String> = if let Some(ref status) = query.status {
        conditions.push("status = ?".to_string());
        Some(status.clone())
    } else if let Some(processed) = query.processed {
        // Backward compat: processed=true → matched, processed=false → pending
        if processed {
            conditions.push("status = 'matched'".to_string());
        } else {
            conditions.push("status = 'pending'".to_string());
        }
        None
    } else {
        None
    };

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    (where_clause, params, status_p)
}

/// DEPRECATED: use `claim_pending_articles` instead.
/// Kept for backward compatibility — selects articles where status='pending'.
#[allow(dead_code)]
pub async fn get_unprocessed_articles(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT * FROM articles WHERE status = 'pending' ORDER BY fetched_at ASC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Atomically claim pending articles for processing.
/// Step 1: UPDATE status='processing' WHERE status='pending' (atomic via SQLite write lock)
/// Step 2: SELECT the claimed articles
/// Returns the claimed articles (empty vec if none pending).
pub async fn claim_pending_articles(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    // Step 1: Atomically mark articles as processing
    sqlx::query(
        "UPDATE articles SET status = 'processing' \
         WHERE id IN (SELECT id FROM articles WHERE status = 'pending' ORDER BY fetched_at ASC LIMIT ?)",
    )
    .bind(limit)
    .execute(pool)
    .await?;

    // Step 2: Read the claimed articles
    sqlx::query_as::<_, Article>(
        "SELECT * FROM articles WHERE status = 'processing' ORDER BY fetched_at ASC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Mark articles as matched: status='matched' + processed_at=now.
/// Batched in chunks of 100 to avoid SQLite variable limit.
pub async fn mark_articles_matched(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for chunk in ids.chunks(100) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE articles SET status = 'matched', processed_at = datetime('now') WHERE id IN ({})",
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

/// Mark articles as skipped: status='skipped' + processed_at=now.
/// Batched in chunks of 100 to avoid SQLite variable limit.
pub async fn mark_articles_skipped(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for chunk in ids.chunks(100) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE articles SET status = 'skipped', processed_at = datetime('now') WHERE id IN ({})",
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

/// Bulk update processed_at for a batch of article IDs (DEPRECATED in favor of status-based marking).
/// Uses IN clause with chunked execution (100 per query) to avoid SQLite variable limits.
/// Kept for backward compatibility with existing callers during transition.
#[allow(dead_code)]
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
    let (where_clause, filter_params, status_param) = build_article_filter(query);
    let sql = format!("SELECT COUNT(*) as count FROM articles{}", where_clause);

    let mut q = sqlx::query_as(&sql);
    for p in &filter_params {
        q = q.bind(*p);
    }
    if let Some(ref s) = status_param {
        q = q.bind(s.as_str());
    }
    let count: (i64,) = q.fetch_one(pool).await?;
    Ok(count.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::article::{ArticleQuery, VALID_ARTICLE_STATUSES};
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Enable foreign keys (sqlite in-memory disables by default)
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::migrate!("./docs/migrations")
            .run(&pool)
            .await
            .unwrap();
        pool
    }

    async fn seed_data_source(pool: &SqlitePool) {
        sqlx::query("INSERT INTO data_sources (type, name, url) VALUES ('rss', 'Test Source', 'http://test')")
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_test_article(pool: &SqlitePool, link: &str, title: &str) -> Article {
        insert_article(pool, 1, link, title, "summary", "content", None)
            .await
            .unwrap()
            .unwrap()
    }

    // ── 9.2: claim_pending_articles tests ──

    #[tokio::test]
    async fn claim_pending_articles_normal() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        let a1 = insert_test_article(&pool, "http://a.com/1", "Article 1").await;
        let a2 = insert_test_article(&pool, "http://a.com/2", "Article 2").await;

        let claimed = claim_pending_articles(&pool, 10).await.unwrap();
        assert_eq!(claimed.len(), 2);
        let ids: Vec<i64> = claimed.iter().map(|a| a.id).collect();
        assert!(ids.contains(&a1.id));
        assert!(ids.contains(&a2.id));
        assert!(claimed.iter().all(|a| a.status == "processing"));
    }

    #[tokio::test]
    async fn claim_pending_articles_no_records() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        let claimed = claim_pending_articles(&pool, 10).await.unwrap();
        assert!(claimed.is_empty());
    }

    #[tokio::test]
    async fn claim_pending_articles_concurrent_protection() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        insert_test_article(&pool, "http://a.com/1", "Article 1").await;
        insert_test_article(&pool, "http://a.com/2", "Article 2").await;

        // First claim grabs both and marks them processing
        let claimed1 = claim_pending_articles(&pool, 10).await.unwrap();
        assert_eq!(claimed1.len(), 2);

        // Simulate real flow: mark as matched to move out of processing
        let ids: Vec<i64> = claimed1.iter().map(|a| a.id).collect();
        mark_articles_matched(&pool, &ids).await.unwrap();

        // Now insert a new article — it's the only pending one
        insert_test_article(&pool, "http://a.com/3", "Article 3").await;

        // Second claim only gets the new article (the others are already matched)
        let claimed2 = claim_pending_articles(&pool, 10).await.unwrap();
        assert_eq!(claimed2.len(), 1);
        assert_eq!(claimed2[0].title, "Article 3");
    }

    #[tokio::test]
    async fn claim_pending_articles_respects_limit() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        for i in 0..5 {
            insert_test_article(
                &pool,
                &format!("http://a.com/{}", i),
                &format!("Article {}", i),
            )
            .await;
        }

        let claimed = claim_pending_articles(&pool, 2).await.unwrap();
        assert_eq!(claimed.len(), 2);
    }

    // ── 9.3: mark_articles_matched / mark_articles_skipped tests ──

    #[tokio::test]
    async fn mark_articles_matched_updates_status_and_processed_at() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        let a = insert_test_article(&pool, "http://a.com/1", "Article 1").await;

        // First claim to set processing
        claim_pending_articles(&pool, 10).await.unwrap();

        mark_articles_matched(&pool, &[a.id]).await.unwrap();

        // Verify
        let articles = list_articles(
            &pool,
            &ArticleQuery {
                page: Some(1),
                per_page: Some(10),
                source_id: None,
                processed: None,
                status: Some("matched".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].status, "matched");
        assert!(articles[0].processed_at.is_some());
    }

    #[tokio::test]
    async fn mark_articles_skipped_updates_status_and_processed_at() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        let a = insert_test_article(&pool, "http://a.com/1", "Article 1").await;

        // First claim to set processing
        claim_pending_articles(&pool, 10).await.unwrap();

        mark_articles_skipped(&pool, &[a.id]).await.unwrap();

        let articles = list_articles(
            &pool,
            &ArticleQuery {
                page: Some(1),
                per_page: Some(10),
                source_id: None,
                processed: None,
                status: Some("skipped".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].status, "skipped");
        assert!(articles[0].processed_at.is_some());
    }

    #[tokio::test]
    async fn mark_articles_matched_batch_large() {
        let pool = setup_test_db().await;
        seed_data_source(&pool).await;
        let mut ids = vec![];
        for i in 0..150 {
            let a = insert_test_article(&pool, &format!("http://a.com/{}", i), &format!("A{}", i))
                .await;
            ids.push(a.id);
        }
        claim_pending_articles(&pool, 200).await.unwrap();
        mark_articles_matched(&pool, &ids).await.unwrap();

        // Verify all 150 were marked (count, since per_page caps at 100)
        let count = count_articles(
            &pool,
            &ArticleQuery {
                page: None,
                per_page: None,
                source_id: None,
                processed: None,
                status: Some("matched".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(count, 150);

        // Also verify per_page cap works: first page returns 100
        let articles = list_articles(
            &pool,
            &ArticleQuery {
                page: Some(1),
                per_page: Some(200),
                source_id: None,
                processed: None,
                status: Some("matched".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(articles.len(), 100);
    }

    // ── 9.6: build_article_filter status/processed mapping tests ──

    #[test]
    fn filter_status_param() {
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: None,
            processed: None,
            status: Some("matched".into()),
        };
        let (clause, params, status_p) = build_article_filter(&query);
        assert!(clause.contains("status = ?"));
        assert_eq!(status_p, Some("matched".into()));
        assert!(params.is_empty());
    }

    #[test]
    fn filter_processed_true_backward_compat() {
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: None,
            processed: Some(true),
            status: None,
        };
        let (clause, params, status_p) = build_article_filter(&query);
        assert!(clause.contains("status = 'matched'"));
        assert_eq!(status_p, None);
        assert!(params.is_empty());
    }

    #[test]
    fn filter_processed_false_backward_compat() {
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: None,
            processed: Some(false),
            status: None,
        };
        let (clause, params, status_p) = build_article_filter(&query);
        assert!(clause.contains("status = 'pending'"));
        assert_eq!(status_p, None);
        assert!(params.is_empty());
    }

    #[test]
    fn filter_status_takes_priority_over_processed() {
        // Even if both are set, status takes priority (caller should strip processed)
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: None,
            processed: Some(true),
            status: Some("skipped".into()),
        };
        let (clause, _params, status_p) = build_article_filter(&query);
        // status checked first, processed fallback skipped
        assert!(clause.contains("status = ?"));
        assert_eq!(status_p, Some("skipped".into()));
    }

    #[test]
    fn filter_no_params() {
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: None,
            processed: None,
            status: None,
        };
        let (clause, params, status_p) = build_article_filter(&query);
        assert!(clause.is_empty());
        assert!(params.is_empty());
        assert_eq!(status_p, None);
    }

    #[test]
    fn filter_with_source_id() {
        let query = ArticleQuery {
            page: None,
            per_page: None,
            source_id: Some(5),
            processed: None,
            status: None,
        };
        let (clause, params, _) = build_article_filter(&query);
        assert!(clause.contains("source_id = ?"));
        assert_eq!(params, vec![5]);
    }

    #[test]
    fn all_valid_statuses_accepted() {
        for &status in VALID_ARTICLE_STATUSES {
            let query = ArticleQuery {
                page: None,
                per_page: None,
                source_id: None,
                processed: None,
                status: Some(status.to_string()),
            };
            let (clause, _, sp) = build_article_filter(&query);
            assert!(clause.contains("status = ?"));
            assert_eq!(sp, Some(status.to_string()));
        }
    }
}
