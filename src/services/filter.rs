use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::config::FilterConfig;
use crate::db;
use crate::pipeline::{Pipeline, PipelineEvent};

mod detection;
mod matching;
mod traits;
mod types;
mod validation;

use matching::AhoCorasickMatcher;
use traits::KeywordMatcher;

// Re-export compute_stats — it is part of the public API.
#[allow(unused_imports)]
pub use detection::compute_stats;

/// Run one filter iteration: load unprocessed articles, match keywords,
/// detect hotspots via burst detection, create push records, mark processed.
///
/// Returns `true` if at least one new push record was created, so the caller
/// can signal downstream modules.
///
/// Shared by the background loop and manual trigger endpoint.
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> bool {
    let (articles, keywords) = match validation::claim_and_validate(pool, config).await {
        Some(v) => v,
        None => return false,
    };

    let matcher = AhoCorasickMatcher::new(&keywords);
    let matches = matcher.match_batch(&articles);

    // Aggregate results from the trait-based matcher output
    let mut hourly_counts: HashMap<i64, i32> = HashMap::new();
    let mut mentions: Vec<(i64, i64)> = Vec::new();
    let mut matched_ids: Vec<i64> = Vec::new();

    for m in &matches {
        matched_ids.push(m.article_id);
        for &kw_id in &m.matched_keyword_ids {
            *hourly_counts.entry(kw_id).or_insert(0) += 1;
            mentions.push((kw_id, m.article_id));
        }
    }

    let matched_set: std::collections::HashSet<i64> = matched_ids.iter().copied().collect();
    let skipped_ids: Vec<i64> = articles
        .iter()
        .map(|a| a.id)
        .filter(|id| !matched_set.contains(id))
        .collect();

    let current_hour = Utc::now().format("%Y%m%d%H").to_string();

    // Persist keyword mentions
    if !mentions.is_empty() {
        match db::keyword_mention::batch_insert_keyword_mentions(pool, &mentions).await {
            Ok(_) => tracing::info!("Filter: inserted {} keyword mention(s)", mentions.len()),
            Err(e) => tracing::error!("Filter: batch insert keyword_mentions failed: {}", e),
        }
    }

    let (created_push, hotspot_count) =
        detection::detect_and_push(pool, config, &keywords, &hourly_counts, &current_hour).await;

    // Mark articles as matched/skipped (after successful commit)
    if !matched_ids.is_empty() {
        if let Err(e) = db::article::mark_articles_matched(pool, &matched_ids).await {
            tracing::error!(
                "Filter: failed to mark {} articles matched: {}",
                matched_ids.len(),
                e
            );
        }
    }
    if !skipped_ids.is_empty() {
        if let Err(e) = db::article::mark_articles_skipped(pool, &skipped_ids).await {
            tracing::error!(
                "Filter: failed to mark {} articles skipped: {}",
                skipped_ids.len(),
                e
            );
        }
    }

    tracing::info!(
        "Filter run complete: {} article(s) processed ({} matched, {} skipped), {} mention(s), {} hotspot(s) created, push_triggered={}",
        articles.len(),
        matched_ids.len(),
        skipped_ids.len(),
        mentions.len(),
        hotspot_count,
        created_push
    );

    created_push
}

/// Background filter loop — runs `run_filter_once` on a configurable interval,
/// or immediately when the Parser signals new articles via `articles_rx`.
pub async fn start_filter_loop(
    pool: SqlitePool,
    config: FilterConfig,
    pipeline: Pipeline,
    mut articles_rx: mpsc::Receiver<PipelineEvent>,
) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval_seconds));

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Filter: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                if run_filter_once(&pool, &config).await {
                    let _ = pipeline.push_ready_tx.try_send(PipelineEvent::NewData);
                }
            }
            Some(_) = articles_rx.recv() => {
                if run_filter_once(&pool, &config).await {
                    let _ = pipeline.push_ready_tx.try_send(PipelineEvent::NewData);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compute_stats;

    #[test]
    fn empty_slice_returns_zero() {
        let (mean, stddev) = compute_stats(&[]);
        assert_eq!(mean, 0.0);
        assert_eq!(stddev, 0.0);
    }

    #[test]
    fn single_element() {
        let (mean, stddev) = compute_stats(&[5]);
        assert!((mean - 5.0).abs() < 0.01);
        assert!((stddev - 0.0).abs() < 0.01);
    }

    #[test]
    fn normal_data() {
        let (mean, stddev) = compute_stats(&[2, 4, 4, 4, 5, 5, 7, 9]);
        assert!((mean - 5.0).abs() < 0.01);
        assert!((stddev - 2.0).abs() < 0.01);
    }
}
