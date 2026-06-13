use sqlx::SqlitePool;

use crate::config::FilterConfig;
use crate::db;
use crate::models::article::Article;
use crate::models::keyword::Keyword;

/// Claim pending articles and validate that keywords are available.
/// Returns `None` if articles are empty, keywords fail to load, or keywords are empty.
pub(super) async fn claim_and_validate(
    pool: &SqlitePool,
    config: &FilterConfig,
) -> Option<(Vec<Article>, Vec<Keyword>)> {
    let articles = match db::article::claim_pending_articles(pool, config.batch_size as i64).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Filter: failed to claim pending articles: {}", e);
            return None;
        }
    };

    if articles.is_empty() {
        return None;
    }
    tracing::info!("Filter: claimed {} pending article(s)", articles.len());

    let keywords = match db::keyword::list_enabled_keywords(pool).await {
        Ok(kws) => kws,
        Err(e) => {
            tracing::error!("Filter: failed to load keywords: {}", e);
            let ids: Vec<i64> = articles.iter().map(|a| a.id).collect();
            if let Err(e) = db::article::mark_articles_skipped(pool, &ids).await {
                tracing::error!("Filter: failed to mark articles skipped: {}", e);
            }
            return None;
        }
    };

    if keywords.is_empty() {
        tracing::warn!(
            "Filter: no enabled keywords found — marking {} article(s) as skipped",
            articles.len()
        );
        let ids: Vec<i64> = articles.iter().map(|a| a.id).collect();
        if let Err(e) = db::article::mark_articles_skipped(pool, &ids).await {
            tracing::error!("Filter: failed to mark articles skipped: {}", e);
        }
        return None;
    }

    Some((articles, keywords))
}
