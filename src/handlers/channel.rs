use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::channel::{CreateChannelRequest, PushChannel, UpdateChannelRequest};
use crate::routes::AppState;

/// GET /api/v1/channels — List all push channels
///
/// Returns all push channels ordered by id ASC.
pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let channels: Vec<PushChannel> = db::channel::list_channels(&state.pool).await?;
    Ok(ApiResponse::ok(channels))
}

/// POST /api/v1/channels — Create a new push channel
///
/// Required: name, config (JSON string). Optional: channel_type (default "webhook").
/// Returns HTTP 201 with the created PushChannel.
pub async fn create_channel(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name must not be empty".into()));
    }
    if req.config.trim().is_empty() {
        return Err(AppError::BadRequest("config must not be empty".into()));
    }
    if serde_json::from_str::<serde_json::Value>(&req.config).is_err() {
        return Err(AppError::BadRequest("config must be valid JSON".into()));
    }

    let channel: PushChannel = db::channel::create_channel(&state.pool, &req).await?;
    Ok(ApiResponse::created(channel))
}

/// POST /api/v1/channels/{id}/update — Update a push channel
///
/// All fields optional — only provided fields are updated.
/// Returns HTTP 200 with the updated PushChannel, or 404 if not found.
pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("name must not be empty".into()));
        }
    }
    if let Some(ref config) = req.config {
        if config.trim().is_empty() {
            return Err(AppError::BadRequest("config must not be empty".into()));
        }
        if serde_json::from_str::<serde_json::Value>(config).is_err() {
            return Err(AppError::BadRequest("config must be valid JSON".into()));
        }
    }

    // Verify the channel exists
    let _existing = db::channel::get_channel_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Channel {} not found", id)))?;

    let updated = db::channel::update_channel(&state.pool, id, &req)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Channel {} not found", id)))?;

    Ok(ApiResponse::ok(updated))
}

/// POST /api/v1/channels/{id}/delete — Delete a push channel
///
/// Returns HTTP 204 on success, or 404 if the channel does not exist.
pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Verify existence first
    let exists = db::channel::get_channel_by_id(&state.pool, id).await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("Channel {} not found", id)));
    }

    db::channel::delete_channel(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
