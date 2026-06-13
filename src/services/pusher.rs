use chrono::Utc;
use futures::stream::StreamExt;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::PusherConfig;
use crate::db;
use crate::pipeline::{Pipeline, PipelineEvent};

/// Run one pusher iteration: recover stale processing records, atomically claim
/// pending records, poll processing and retry-due records, send webhooks,
/// update status with exponential backoff.
///
/// Shared by the background loop and manual trigger endpoint.
pub async fn run_pusher_once(pool: &SqlitePool, config: &PusherConfig, client: &reqwest::Client) {
    // 0. Recover stale processing records (crashed previous run)
    match db::push_record::recover_stale_processing_records(pool, config.stale_timeout_minutes)
        .await
    {
        Ok(n) => {
            if n > 0 {
                tracing::info!("Pusher: recovered {} stale processing record(s)", n);
            }
        }
        Err(e) => {
            tracing::error!("Pusher: failed to recover stale records: {}", e);
        }
    }

    // 1. Atomically claim pending records
    if let Err(e) = db::push_record::claim_pending_records(pool).await {
        tracing::error!("Pusher: failed to claim pending records: {}", e);
        return;
    }
    tracing::debug!("Pusher: claim_pending_records completed");

    // 2. Poll processing records (atomically claimed) and retry-due records
    let processing = match db::push_record::list_processing_records(pool).await {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Pusher: failed to list processing records: {}", e);
            return;
        }
    };

    let retry_due = match db::push_record::list_retry_due_records(pool, config.max_retries).await {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Pusher: failed to list retry-due records: {}", e);
            return;
        }
    };

    let pushable: Vec<_> = processing.into_iter().chain(retry_due).collect();

    if pushable.is_empty() {
        return;
    }

    tracing::info!("Pusher: {} pushable record(s)", pushable.len());

    // 3. Process each record concurrently (max 8 concurrent)
    let success_cnt = Arc::new(AtomicU32::new(0));
    let failed_cnt = Arc::new(AtomicU32::new(0));
    let gave_up_cnt = Arc::new(AtomicU32::new(0));

    futures::stream::iter(&pushable)
        .for_each_concurrent(8, |record| {
            let pool = pool.clone();
            let config = config.clone();
            let client = client.clone();
            let sc = Arc::clone(&success_cnt);
            let fc = Arc::clone(&failed_cnt);
            let gc = Arc::clone(&gave_up_cnt);
            async move {
                process_one(&pool, &config, &client, record, &sc, &fc, &gc).await;
            }
        })
        .await;

    tracing::info!(
        "Pusher batch complete: {} success, {} failed, {} gave up",
        success_cnt.load(Ordering::Relaxed),
        failed_cnt.load(Ordering::Relaxed),
        gave_up_cnt.load(Ordering::Relaxed)
    );
}

/// Process a single push record: lookup channel & event, POST webhook, update status.
async fn process_one(
    pool: &SqlitePool,
    config: &PusherConfig,
    client: &reqwest::Client,
    record: &crate::models::push_record::PushRecord,
    success_cnt: &AtomicU32,
    failed_cnt: &AtomicU32,
    gave_up_cnt: &AtomicU32,
) {
    // 2a. Lookup channel config
    let channel = match db::channel::get_channel_by_id(pool, record.channel_id).await {
        Ok(Some(ch)) => ch,
        Ok(None) => {
            tracing::error!(
                "Pusher: channel {} not found for record {}",
                record.channel_id,
                record.id
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                "Pusher: failed to lookup channel {}: {}",
                record.channel_id,
                e
            );
            return;
        }
    };

    // 2b. Lookup hot event
    let hot_event = match db::hot_event::get_hot_event_by_id(pool, record.hot_event_id).await {
        Ok(Some(he)) => he,
        Ok(None) => {
            tracing::error!(
                "Pusher: hot_event {} not found for record {}",
                record.hot_event_id,
                record.id
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                "Pusher: failed to lookup hot_event {}: {}",
                record.hot_event_id,
                e
            );
            return;
        }
    };

    // 2c. Lookup keyword
    let keyword = match db::keyword::get_keyword_by_id(pool, hot_event.keyword_id).await {
        Ok(Some(kw)) => kw,
        Ok(None) => {
            tracing::error!(
                "Pusher: keyword {} not found for hot_event {}",
                hot_event.keyword_id,
                hot_event.id
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                "Pusher: failed to lookup keyword {}: {}",
                hot_event.keyword_id,
                e
            );
            return;
        }
    };

    // 2d. Extract webhook URL from channel config JSON
    let webhook_url = match extract_webhook_url(&channel.config) {
        Some(url) => url,
        None => {
            tracing::error!(
                "Pusher: channel {} '{}' has no valid url in config",
                channel.id,
                channel.name
            );
            // Mark as failed with max retries (skip) — no valid webhook URL
            let err_msg = format!("Channel {} has no valid webhook URL", channel.id);
            if mark_failed(pool, config, record.id, record.retry_count, &err_msg).await {
                gave_up_cnt.fetch_add(1, Ordering::Relaxed);
            } else {
                failed_cnt.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
    };

    // 2e. Build payload
    let payload = serde_json::json!({
        "msgtype": "text",
        "text": {
            "content": format!(
                "🔥 Hotspot Alert\nKeyword: {}\nCount: {}\nHour: {}\nMean: {:.2}\nStdDev: {:.2}",
                keyword.word,
                hot_event.count,
                hot_event.hour_bucket,
                hot_event.mean_historical,
                hot_event.stddev_historical
            )
        }
    });

    // 2f. POST webhook
    match client.post(&webhook_url).json(&payload).send().await {
        Ok(response) => {
            if response.status().is_success() {
                tracing::info!(
                    "Pusher: record {} sent to channel '{}' — success (status={})",
                    record.id,
                    channel.name,
                    response.status()
                );
                success_cnt.fetch_add(1, Ordering::Relaxed);
                // Mark success with optimistic locking
                let updated = db::push_record::update_push_status_optimistic(
                    pool,
                    record.id,
                    &record.status, // expected current status
                    "success",
                    record.retry_count,
                    None,
                    None, // last_error: None on success
                )
                .await;

                match updated {
                    Ok(true) => {}
                    Ok(false) => {
                        tracing::warn!(
                            "Pusher: record {} was already updated by another process, skipping",
                            record.id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Pusher: failed to update record {} status: {}",
                            record.id,
                            e
                        );
                    }
                }
            } else {
                tracing::warn!(
                    "Pusher: record {} to channel '{}' failed — status={}",
                    record.id,
                    channel.name,
                    response.status()
                );
                let err_msg = format!("HTTP {}", response.status());
                if mark_failed(pool, config, record.id, record.retry_count, &err_msg).await {
                    gave_up_cnt.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_cnt.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        Err(e) => {
            tracing::error!(
                "Pusher: record {} to channel '{}' — network error: {}",
                record.id,
                channel.name,
                e
            );
            let err_msg = format!("Network error: {}", e);
            if mark_failed(pool, config, record.id, record.retry_count, &err_msg).await {
                gave_up_cnt.fetch_add(1, Ordering::Relaxed);
            } else {
                failed_cnt.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Mark a push record as failed with exponential backoff.
/// `next_retry_at = now + min(base * 2^(n-1), retry_max_seconds)`
/// If max retries exhausted, marks the record as `dead` (terminal state).
/// Returns `true` if the record reached terminal state (dead or gave up).
async fn mark_failed(
    pool: &SqlitePool,
    config: &PusherConfig,
    record_id: i64,
    current_retry_count: i32,
    last_error: &str,
) -> bool {
    let new_retry_count = current_retry_count + 1;
    let gave_up = new_retry_count >= config.max_retries as i32;

    if gave_up {
        tracing::warn!(
            "Pusher: record {} reached max retries ({}), marking as dead. Last error: {}",
            record_id,
            config.max_retries,
            last_error
        );
        // Terminal dead state — no next_retry_at
        if let Err(e) = db::push_record::update_push_status(
            pool,
            record_id,
            "dead",
            new_retry_count,
            None,
            Some(last_error),
        )
        .await
        {
            tracing::error!("Pusher: failed to mark record {} as dead: {}", record_id, e);
        }
        return true;
    }

    // Exponential backoff: delay = min(base * 2^(new_retry_count-1), retry_max_seconds)
    let delay = compute_backoff_delay(
        config.retry_base_seconds,
        config.retry_max_seconds,
        new_retry_count,
    );
    let next_retry_at = Some(Utc::now().naive_utc() + chrono::Duration::seconds(delay));

    if let Err(e) = db::push_record::update_push_status(
        pool,
        record_id,
        "failed",
        new_retry_count,
        next_retry_at,
        Some(last_error),
    )
    .await
    {
        tracing::error!(
            "Pusher: failed to update record {} status: {}",
            record_id,
            e
        );
    }
    false
}

/// Compute exponential backoff delay for retry_count N:
///   delay = min(base * 2^(N-1), retry_max_seconds)
///
/// Examples (base=60, max=3600):
///   N=1 → 60, N=2 → 120, N=3 → 240, N=4 → 480, N=5 → 960, N=7 → 3600 (capped)
pub fn compute_backoff_delay(base_seconds: u64, max_seconds: u64, retry_count: i32) -> i64 {
    if retry_count <= 0 {
        return base_seconds as i64;
    }
    let shift = 1i64.checked_shl(retry_count as u32 - 1).unwrap_or(i64::MAX);
    let exp_delay = (base_seconds as i64).saturating_mul(shift);
    exp_delay.min(max_seconds as i64)
}

/// Extract the webhook URL from a channel's config JSON.
/// Returns None if the config does not contain a valid `url` field.
fn extract_webhook_url(config_json: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(config_json).ok()?;
    parsed.get("url")?.as_str().map(|s| s.to_string())
}

/// Background pusher loop — runs `run_pusher_once` on a configurable interval,
/// or immediately when the Filter signals new push records via `push_rx`.
pub async fn start_pusher_loop(
    pool: SqlitePool,
    config: PusherConfig,
    pipeline: Pipeline,
    mut push_rx: mpsc::Receiver<PipelineEvent>,
) {
    let client = reqwest::Client::new();
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval_seconds));

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Pusher: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                run_pusher_once(&pool, &config, &client).await;
            }
            Some(_) = push_rx.recv() => {
                run_pusher_once(&pool, &config, &client).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 9.4: compute_backoff_delay tests ──

    #[test]
    fn backoff_retry_1_is_base() {
        assert_eq!(compute_backoff_delay(60, 3600, 1), 60);
    }

    #[test]
    fn backoff_retry_2_is_double() {
        assert_eq!(compute_backoff_delay(60, 3600, 2), 120);
    }

    #[test]
    fn backoff_retry_3_is_quadruple() {
        assert_eq!(compute_backoff_delay(60, 3600, 3), 240);
    }

    #[test]
    fn backoff_retry_4() {
        assert_eq!(compute_backoff_delay(60, 3600, 4), 480);
    }

    #[test]
    fn backoff_retry_5() {
        assert_eq!(compute_backoff_delay(60, 3600, 5), 960);
    }

    #[test]
    fn backoff_retry_6() {
        assert_eq!(compute_backoff_delay(60, 3600, 6), 1920);
    }

    #[test]
    fn backoff_capped_at_max() {
        // 60 * 2^6 = 60 * 64 = 3840, capped at 3600
        assert_eq!(compute_backoff_delay(60, 3600, 7), 3600);
    }

    #[test]
    fn backoff_high_retry_still_capped() {
        // 60 * 2^9 = 60 * 512 = 30720, capped at 3600
        assert_eq!(compute_backoff_delay(60, 3600, 10), 3600);
    }

    #[test]
    fn backoff_zero_retry_returns_base() {
        assert_eq!(compute_backoff_delay(60, 3600, 0), 60);
    }

    #[test]
    fn backoff_different_base() {
        assert_eq!(compute_backoff_delay(30, 3600, 1), 30);
        assert_eq!(compute_backoff_delay(30, 3600, 2), 60);
        assert_eq!(compute_backoff_delay(30, 3600, 3), 120);
    }

    #[test]
    fn backoff_small_max() {
        assert_eq!(compute_backoff_delay(60, 100, 1), 60);
        assert_eq!(compute_backoff_delay(60, 100, 2), 100); // 120 capped at 100
        assert_eq!(compute_backoff_delay(60, 100, 3), 100); // 240 capped at 100
    }
}
