use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::article::{ArticleQuery, VALID_ARTICLE_STATUSES};
use crate::pipeline::PipelineEvent;
use crate::routes::AppState;

/// Generic paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

// ── Query param structs ──

#[derive(Debug, Deserialize)]
pub struct ArticleListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_id: Option<i64>,
    /// DEPRECATED: use `status` instead. Backward compat: true→matched, false→pending.
    pub processed: Option<bool>,
    /// Filter by article status: pending | processing | matched | skipped
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HotspotListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub keyword_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TrendParams {
    pub hours: Option<i64>,
}

// ── Handlers ──

/// GET /api/v1/articles — list articles with pagination and optional filtering
pub async fn list_articles(
    State(state): State<AppState>,
    Query(params): Query<ArticleListParams>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    // Validate status if provided
    let status = match &params.status {
        Some(s) => {
            if !VALID_ARTICLE_STATUSES.contains(&s.as_str()) {
                return Err(AppError::InvalidStatus(s.clone()));
            }
            Some(s.clone())
        }
        None => None,
    };

    // If both status and processed are provided, status takes priority
    // (processed is only used as fallback when status is None — handled in build_article_filter)
    let processed = if status.is_some() {
        None
    } else {
        params.processed
    };

    let query = ArticleQuery {
        page: Some(page),
        per_page: Some(per_page),
        source_id: params.source_id,
        processed,
        status,
    };

    let items = db::article::list_articles(&state.pool, &query).await?;
    let total = db::article::count_articles(&state.pool, &query).await?;

    let resp = PaginatedResponse {
        items,
        total,
        page,
        per_page,
    };
    Ok(ApiResponse::ok(resp))
}

/// GET /api/v1/hotspots — list hotspots with pagination and optional keyword filter
pub async fn list_hotspots(
    State(state): State<AppState>,
    Query(params): Query<HotspotListParams>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;

    let items =
        db::hot_event::list_hotspots_paginated(&state.pool, params.keyword_id, limit, offset)
            .await?;
    let total = db::hot_event::count_hotspots(&state.pool, params.keyword_id).await?;

    let resp = PaginatedResponse {
        items,
        total,
        page,
        per_page,
    };
    Ok(ApiResponse::ok(resp))
}

/// GET /api/v1/hotspots/{id}/push-records — list push records for a hotspot
pub async fn get_push_records(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let records = db::push_record::list_push_records_with_details(&state.pool, id).await?;
    Ok(ApiResponse::ok(records))
}

/// Trend data point in response
#[derive(Debug, Serialize)]
pub struct TrendPoint {
    pub hour_bucket: String,
    pub count: i32,
}

/// Trend response with keyword metadata
#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub keyword_id: i64,
    pub keyword: String,
    pub points: Vec<TrendPoint>,
}

/// GET /api/v1/trend/{keyword_id} — get hourly trend data for a keyword
pub async fn get_trend(
    State(state): State<AppState>,
    Path(keyword_id): Path<i64>,
    Query(params): Query<TrendParams>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Verify keyword exists
    let keyword = db::keyword::get_keyword_by_id(&state.pool, keyword_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Keyword {} not found", keyword_id)))?;

    let hours = params.hours.unwrap_or(24).clamp(1, 8760);
    let rows = db::hot_event::get_hourly_counts(&state.pool, keyword_id, hours as i32).await?;

    let points: Vec<TrendPoint> = rows
        .into_iter()
        .map(|(hour_bucket, count)| TrendPoint { hour_bucket, count })
        .collect();

    let resp = TrendResponse {
        keyword_id,
        keyword: keyword.word,
        points,
    };
    Ok(ApiResponse::ok(resp))
}

// ── Settings handler ──

/// GET /api/v1/settings — return current server configuration
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok(ApiResponse::ok(&state.config))
}

// ── Trigger handlers ──

/// POST /api/v1/trigger/filter — manually run one filter iteration
pub async fn trigger_filter(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    tracing::info!("Manual trigger: filter run started");
    let created_push =
        crate::services::filter::run_filter_once(&state.pool, &state.config.filter).await;
    if created_push {
        let _ = state
            .pipeline
            .push_ready_tx
            .try_send(PipelineEvent::NewData);
    }
    Ok(ApiResponse::ok(json!({"message": "Filter executed"})))
}

/// POST /api/v1/trigger/pusher — manually run one pusher iteration
pub async fn trigger_pusher(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    tracing::info!("Manual trigger: pusher run started");
    let client = reqwest::Client::new();
    crate::services::pusher::run_pusher_once(&state.pool, &state.config.pusher, &client).await;
    Ok(ApiResponse::ok(json!({"message": "Pusher executed"})))
}
