use chrono::Utc;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

use crate::config::PusherConfig;
use crate::db;
use crate::pipeline::{Pipeline, PipelineEvent};

/// Run one pusher iteration: poll pushable records, send webhooks,
/// update status with exponential backoff.
///
/// Shared by the background loop and manual trigger endpoint.
pub async fn run_pusher_once(pool: &SqlitePool, config: &PusherConfig) {
    // 1. Poll pending and retry-due records
    let pending = match db::push_record::list_pending_records(pool).await {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Pusher: failed to list pending records: {}", e);
            return;
        }
    };

    let retry_due = match db::push_record::list_retry_due_records(pool).await {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Pusher: failed to list retry-due records: {}", e);
            return;
        }
    };

    let pushable: Vec<_> = pending.into_iter().chain(retry_due).collect();

    if pushable.is_empty() {
        return;
    }

    tracing::info!("Pusher: {} pushable record(s)", pushable.len());

    // 2. Process each record
    let client = reqwest::Client::new();

    for record in &pushable {
        process_one(pool, config, &client, record).await;
    }
}

/// Process a single push record: lookup channel & event, POST webhook, update status.
async fn process_one(
    pool: &SqlitePool,
    config: &PusherConfig,
    client: &reqwest::Client,
    record: &crate::models::push_record::PushRecord,
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
            // Mark as failed with max retries (skip)
            let _ = mark_failed(pool, config, record.id, record.retry_count).await;
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
                // Mark success with optimistic locking
                let updated = db::push_record::update_push_status_optimistic(
                    pool,
                    record.id,
                    &record.status, // expected current status
                    "success",
                    record.retry_count,
                    None,
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
                mark_failed(pool, config, record.id, record.retry_count).await;
            }
        }
        Err(e) => {
            tracing::error!(
                "Pusher: record {} to channel '{}' — network error: {}",
                record.id,
                channel.name,
                e
            );
            mark_failed(pool, config, record.id, record.retry_count).await;
        }
    }
}

/// Mark a push record as failed with exponential backoff.
/// `next_retry_at = now + (retry_count * retry_base_seconds)`
/// If max retries reached, sets `next_retry_at` to NULL.
async fn mark_failed(
    pool: &SqlitePool,
    config: &PusherConfig,
    record_id: i64,
    current_retry_count: i32,
) {
    let new_retry_count = current_retry_count + 1;
    let next_retry_at = if new_retry_count >= config.max_retries as i32 {
        tracing::warn!(
            "Pusher: record {} reached max retries ({}), giving up",
            record_id,
            config.max_retries
        );
        None
    } else {
        let delay = new_retry_count as i64 * config.retry_base_seconds as i64;
        let at = Utc::now().naive_utc() + chrono::Duration::seconds(delay);
        Some(at)
    };

    if let Err(e) = db::push_record::update_push_status(
        pool,
        record_id,
        "failed",
        new_retry_count,
        next_retry_at,
    )
    .await
    {
        tracing::error!(
            "Pusher: failed to update record {} status: {}",
            record_id,
            e
        );
    }
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
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval_seconds));

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Pusher: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                run_pusher_once(&pool, &config).await;
            }
            Some(_) = push_rx.recv() => {
                run_pusher_once(&pool, &config).await;
            }
        }
    }
}
