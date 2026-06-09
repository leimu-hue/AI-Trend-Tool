use aho_corasick::AhoCorasickBuilder;
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::config::FilterConfig;
use crate::db;
use crate::pipeline::{Pipeline, PipelineEvent};

/// Run one filter iteration: load unprocessed articles, match keywords,
/// detect hotspots via burst detection, create push records, mark processed.
///
/// Returns `true` if at least one new push record was created, so the caller
/// can signal downstream modules.
///
/// Shared by the background loop and manual trigger endpoint.
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> bool {
    // 1. Load unprocessed articles
    let articles = match db::article::get_unprocessed_articles(pool, config.batch_size as i64).await
    {
        Ok(articles) => articles,
        Err(e) => {
            tracing::error!("Filter: failed to load unprocessed articles: {}", e);
            return false;
        }
    };

    if articles.is_empty() {
        return false;
    }

    tracing::info!("Filter: loaded {} unprocessed article(s)", articles.len());

    // 2. Load enabled keywords
    let keywords = match db::keyword::list_enabled_keywords(pool).await {
        Ok(kws) => kws,
        Err(e) => {
            tracing::error!("Filter: failed to load keywords: {}", e);
            return false;
        }
    };

    if keywords.is_empty() {
        // No keywords — mark all articles as processed
        let ids: Vec<i64> = articles.iter().map(|a| a.id).collect();
        if let Err(e) = db::article::mark_processed_batch(pool, &ids).await {
            tracing::error!("Filter: failed to mark articles processed: {}", e);
        }
        return false;
    }

    // 3. Build Aho-Corasick automata
    // Separate case-insensitive and case-sensitive keywords
    let ci_kws: Vec<_> = keywords
        .iter()
        .enumerate()
        .filter(|(_, k)| !k.case_sensitive)
        .collect();
    let cs_kws: Vec<_> = keywords
        .iter()
        .enumerate()
        .filter(|(_, k)| k.case_sensitive)
        .collect();

    let ci_patterns: Vec<String> = ci_kws.iter().map(|(_, k)| k.word.to_lowercase()).collect();
    let cs_patterns: Vec<String> = cs_kws.iter().map(|(_, k)| k.word.clone()).collect();

    let ci_ac = if !ci_patterns.is_empty() {
        Some(
            AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(&ci_patterns)
                .expect("Failed to build CI automaton"),
        )
    } else {
        None
    };

    let cs_ac = if !cs_patterns.is_empty() {
        Some(
            AhoCorasickBuilder::new()
                .build(&cs_patterns)
                .expect("Failed to build CS automaton"),
        )
    } else {
        None
    };

    // 4. Match articles and accumulate hourly counts
    let current_hour = Utc::now().format("%Y%m%d%H").to_string();
    let mut hourly_counts: HashMap<i64, i32> = HashMap::new(); // keyword_id -> count
    let mut article_ids: Vec<i64> = Vec::new();

    for article in &articles {
        let text = format!("{} {}", article.title, article.summary);
        article_ids.push(article.id);

        // Case-insensitive matching
        if let Some(ref ac) = ci_ac {
            for mat in ac.find_iter(&text) {
                let (_, kw) = ci_kws[mat.pattern()];
                *hourly_counts.entry(kw.id).or_insert(0) += 1;
                if let Err(e) =
                    db::keyword_mention::insert_keyword_mention(pool, kw.id, article.id).await
                {
                    tracing::error!(
                        "Filter: failed to insert keyword_mention (kw={}, art={}): {}",
                        kw.id,
                        article.id,
                        e
                    );
                }
            }
        }

        // Case-sensitive matching
        if let Some(ref ac) = cs_ac {
            for mat in ac.find_iter(&text) {
                let (_, kw) = cs_kws[mat.pattern()];
                *hourly_counts.entry(kw.id).or_insert(0) += 1;
                if let Err(e) =
                    db::keyword_mention::insert_keyword_mention(pool, kw.id, article.id).await
                {
                    tracing::error!(
                        "Filter: failed to insert keyword_mention (kw={}, art={}): {}",
                        kw.id,
                        article.id,
                        e
                    );
                }
            }
        }
    }

    // 5. Burst detection and hot_event creation
    let enabled_channels = match db::channel::list_enabled_channels(pool).await {
        Ok(chs) => chs,
        Err(e) => {
            tracing::error!("Filter: failed to load channels: {}", e);
            return false;
        }
    };

    let mut created_push = false;

    for kw in &keywords {
        let current_count = *hourly_counts.get(&kw.id).unwrap_or(&0);

        if current_count == 0 {
            continue;
        }

        let (mean, stddev) =
            compute_historical_stats(pool, kw.id, config.history_hours as i32).await;

        // Record the hourly count in hot_events (upsert for idempotency)
        let hot_event =
            match upsert_hot_event_record(pool, kw.id, &current_hour, current_count, mean, stddev)
                .await
            {
                Ok(he) => he,
                Err(e) => {
                    tracing::error!(
                        "Filter: failed to upsert hot_event (kw={}, hour={}): {}",
                        kw.id,
                        current_hour,
                        e
                    );
                    continue;
                }
            };

        // Burst detection: is current_count above threshold?
        let has_enough_history =
            db::hot_event::get_hourly_counts(pool, kw.id, config.min_history_hours as i32)
                .await
                .map(|rows| rows.len() >= config.min_history_hours as usize)
                .unwrap_or(false);

        let threshold = mean + (kw.std_multiplier * stddev);
        let is_hotspot = has_enough_history
            && current_count > threshold as i32
            && current_count >= kw.min_hot_count;

        if is_hotspot {
            tracing::info!(
                "Filter: HOTSPOT detected — keyword '{}' (id={}), count={}, mean={:.2}, stddev={:.2}, threshold={:.2}",
                kw.word,
                kw.id,
                current_count,
                mean,
                stddev,
                threshold
            );

            let channel_ids: Vec<i64> = enabled_channels.iter().map(|c| c.id).collect();
            match db::push_record::insert_push_records_for_event(pool, hot_event.id, &channel_ids)
                .await
            {
                Ok(_) => created_push = true,
                Err(e) => {
                    tracing::error!(
                        "Filter: failed to insert push_records for hot_event {}: {}",
                        hot_event.id,
                        e
                    );
                }
            }
        }
    }

    // 6. Mark articles as processed
    if let Err(e) = db::article::mark_processed_batch(pool, &article_ids).await {
        tracing::error!("Filter: failed to mark articles processed: {}", e);
    }

    created_push
}

/// Compute historical mean and stddev from hot_events hourly counts
async fn compute_historical_stats(
    pool: &SqlitePool,
    keyword_id: i64,
    history_hours: i32,
) -> (f64, f64) {
    let rows = match db::hot_event::get_hourly_counts(pool, keyword_id, history_hours).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(
                "Filter: failed to get historical counts for kw {}: {}",
                keyword_id,
                e
            );
            return (0.0, 0.0);
        }
    };

    if rows.is_empty() {
        return (0.0, 0.0);
    }

    let counts: Vec<f64> = rows.iter().map(|(_, c)| *c as f64).collect();
    let n = counts.len() as f64;
    let mean = counts.iter().sum::<f64>() / n;

    let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    (mean, stddev)
}

/// Upsert a hot_event record: delete existing (keyword_id, hour_bucket), then insert.
async fn upsert_hot_event_record(
    pool: &SqlitePool,
    keyword_id: i64,
    hour_bucket: &str,
    count: i32,
    mean_historical: f64,
    stddev_historical: f64,
) -> Result<crate::models::hot_event::HotEvent, sqlx::Error> {
    // Delete existing record for idempotency (prevents duplicate key+hour rows)
    sqlx::query("DELETE FROM hot_events WHERE keyword_id = ? AND hour_bucket = ?")
        .bind(keyword_id)
        .bind(hour_bucket)
        .execute(pool)
        .await?;

    db::hot_event::insert_hot_event(
        pool,
        keyword_id,
        hour_bucket,
        count,
        mean_historical,
        stddev_historical,
    )
    .await
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
