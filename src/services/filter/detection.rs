use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::config::FilterConfig;
use crate::db;
use crate::models::keyword::Keyword;

/// Compute mean and standard deviation from a slice of hourly counts.
/// Returns (mean, stddev). An empty slice returns (0.0, 0.0).
pub fn compute_stats(counts: &[i32]) -> (f64, f64) {
    if counts.is_empty() {
        return (0.0, 0.0);
    }
    let n = counts.len() as f64;
    let mean = counts.iter().map(|c| *c as f64).sum::<f64>() / n;
    let variance = counts
        .iter()
        .map(|c| (*c as f64 - mean).powi(2))
        .sum::<f64>()
        / n;
    (mean, variance.sqrt())
}

/// Run burst detection, create hot events and push records inside a transaction.
/// Returns (created_push, hotspot_count).
pub(super) async fn detect_and_push(
    pool: &SqlitePool,
    config: &FilterConfig,
    keywords: &[Keyword],
    hourly_counts: &HashMap<i64, i32>,
    current_hour: &str,
) -> (bool, u32) {
    let enabled_channels = match db::channel::list_enabled_channels(pool).await {
        Ok(chs) => chs,
        Err(e) => {
            tracing::error!("Filter: failed to load channels: {}", e);
            return (false, 0);
        }
    };

    let all_counts = match db::hot_event::get_all_hourly_counts(
        pool,
        config.history_hours.max(config.min_history_hours) as i32,
    )
    .await
    {
        Ok(counts) => counts,
        Err(e) => {
            tracing::error!("Filter: failed to load historical counts: {}", e);
            return (false, 0);
        }
    };

    // Group historical counts by keyword
    let mut kw_stats: HashMap<i64, Vec<i32>> = HashMap::new();
    let mut kw_hours: HashMap<i64, usize> = HashMap::new();
    for (kw_id, _hour, count) in &all_counts {
        kw_stats.entry(*kw_id).or_default().push(*count);
        *kw_hours.entry(*kw_id).or_insert(0) += 1;
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Filter: failed to begin transaction: {}", e);
            return (false, 0);
        }
    };

    let mut created_push = false;
    let mut hotspot_count = 0u32;

    for kw in keywords {
        let current_count = *hourly_counts.get(&kw.id).unwrap_or(&0);
        if current_count == 0 {
            continue;
        }

        let (mean, stddev) = kw_stats
            .get(&kw.id)
            .map(|counts| compute_stats(counts))
            .unwrap_or((0.0, 0.0));

        let hot_event = match upsert_hot_event_record(
            &mut tx, kw.id, current_hour, current_count, mean, stddev,
        )
        .await
        {
            Ok(he) => he,
            Err(e) => {
                tracing::error!(
                    "Filter: failed to upsert hot_event (kw={}, hour={}): {}",
                    kw.id, current_hour, e
                );
                continue;
            }
        };

        let has_enough_history = kw_hours
            .get(&kw.id)
            .map(|&h| h >= config.min_history_hours as usize)
            .unwrap_or(false);
        let threshold = mean + (kw.std_multiplier * stddev);
        let is_hotspot = has_enough_history
            && current_count > threshold as i32
            && current_count >= kw.min_hot_count;

        if is_hotspot {
            tracing::info!(
                "Filter: HOTSPOT detected — keyword '{}' (id={}), count={}, mean={:.2}, stddev={:.2}, threshold={:.2}",
                kw.word, kw.id, current_count, mean, stddev, threshold
            );

            let channel_ids: Vec<i64> = enabled_channels.iter().map(|c| c.id).collect();
            match db::push_record::insert_push_records_for_event_tx(
                &mut tx, hot_event.id, &channel_ids,
            )
            .await
            {
                Ok(_) => {
                    created_push = true;
                    hotspot_count += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "Filter: failed to insert push_records for hot_event {}: {}",
                        hot_event.id, e
                    );
                }
            }
        }
    }

    if let Err(e) = tx.commit().await {
        tracing::error!("Filter: failed to commit transaction: {}", e);
        return (false, 0);
    }

    (created_push, hotspot_count)
}

/// Upsert a hot_event record inside a transaction.
async fn upsert_hot_event_record(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    keyword_id: i64,
    hour_bucket: &str,
    count: i32,
    mean_historical: f64,
    stddev_historical: f64,
) -> Result<crate::models::hot_event::HotEvent, sqlx::Error> {
    sqlx::query_as::<_, crate::models::hot_event::HotEvent>(
        "INSERT INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(keyword_id, hour_bucket) DO UPDATE SET \
         count = excluded.count, \
         mean_historical = excluded.mean_historical, \
         stddev_historical = excluded.stddev_historical \
         RETURNING *",
    )
    .bind(keyword_id)
    .bind(hour_bucket)
    .bind(count)
    .bind(mean_historical)
    .bind(stddev_historical)
    .fetch_one(&mut **tx)
    .await
}
