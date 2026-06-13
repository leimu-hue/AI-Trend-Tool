use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use validator::Validate;

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::source::{CreateSourceRequest, DataSource, UpdateSourceRequest};
use crate::routes::AppState;

/// GET /api/v1/sources — List all data sources
///
/// Returns all data sources ordered by created_at DESC.
pub async fn list_sources(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let sources = db::source::list_sources_with_count(&state.pool).await?;
    Ok(ApiResponse::ok(sources))
}

/// POST /api/v1/sources — Create a new data source
///
/// Required fields: type, name, url.
/// Optional: interval_seconds (default 300), config (default "{}").
/// Returns HTTP 201 with the created DataSource.
pub async fn create_source(
    State(state): State<AppState>,
    Json(req): Json<CreateSourceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    req.validate()?;

    let source: DataSource = db::source::create_source(&state.pool, &req).await?;
    Ok(ApiResponse::created(source))
}

/// POST /api/v1/sources/{id}/update — Update a data source
///
/// All fields optional — only provided fields are updated.
/// Returns HTTP 200 with the updated DataSource, or 404 if not found.
pub async fn update_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSourceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("name must not be empty".into()));
        }
    }
    if let Some(ref url) = req.url {
        if url.trim().is_empty() {
            return Err(AppError::BadRequest("url must not be empty".into()));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::BadRequest(
                "url must start with http:// or https://".into(),
            ));
        }
    }

    // Verify the source exists
    let _existing = db::source::get_source_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Source {} not found", id)))?;

    let updated = db::source::update_source(&state.pool, id, &req)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Source {} not found", id)))?;

    Ok(ApiResponse::ok(updated))
}

/// POST /api/v1/sources/{id}/delete — Delete a data source
///
/// Returns HTTP 204 on success, or 404 if the source does not exist.
pub async fn delete_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Verify existence first (hard delete — must check before)
    let exists = db::source::get_source_by_id(&state.pool, id).await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("Source {} not found", id)));
    }

    db::source::delete_source(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/sources/{id}/fetch — Trigger manual fetch for a source
///
/// Resets last_fetched_at to NULL so the Parser picks it up on its next cycle.
/// Returns HTTP 200 with a confirmation message, or 404 if not found.
pub async fn trigger_fetch(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Verify the source exists
    let _existing = db::source::get_source_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Source {} not found", id)))?;

    db::source::reset_last_fetched(&state.pool, id).await?;

    let msg = serde_json::json!({ "message": format!("Fetch triggered for source {}", id) });
    Ok(ApiResponse::ok(msg))
}
