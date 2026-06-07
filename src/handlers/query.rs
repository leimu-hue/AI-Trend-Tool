use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::article::ArticleQuery;
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
    pub processed: Option<bool>,
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
    let query = ArticleQuery {
        page: params.page,
        per_page: params.per_page,
        source_id: params.source_id,
        processed: params.processed,
    };
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

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

    let hours = params.hours.unwrap_or(24);
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

// ── Trigger handlers ──

/// POST /api/v1/trigger/filter — manually run one filter iteration
pub async fn trigger_filter(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    crate::services::filter::run_filter_once(&state.pool, &state.config.filter).await;
    Ok(ApiResponse::ok(json!({"message": "Filter executed"})))
}

/// POST /api/v1/trigger/pusher — manually run one pusher iteration
pub async fn trigger_pusher(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    crate::services::pusher::run_pusher_once(&state.pool, &state.config.pusher).await;
    Ok(ApiResponse::ok(json!({"message": "Pusher executed"})))
}
