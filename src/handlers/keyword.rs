use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use validator::Validate;

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::keyword::{CreateKeywordRequest, Keyword, UpdateKeywordRequest};
use crate::routes::AppState;

/// GET /api/v1/keywords — List all keywords
///
/// Returns all keywords ordered by created_at DESC.
pub async fn list_keywords(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let keywords: Vec<Keyword> = db::keyword::list_keywords(&state.pool).await?;
    Ok(ApiResponse::ok(keywords))
}

/// POST /api/v1/keywords — Create a new keyword
///
/// Required: word. Optional: case_sensitive (default false),
/// std_multiplier (default 2.0), min_hot_count (default 3).
/// Returns HTTP 201 with the created Keyword, or 409 on duplicate word.
pub async fn create_keyword(
    State(state): State<AppState>,
    Json(req): Json<CreateKeywordRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    req.validate()?;

    let keyword: Keyword = db::keyword::create_keyword(&state.pool, &req)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
                AppError::Conflict(format!("Keyword '{}' already exists", req.word))
            }
            _ => AppError::from(e),
        })?;

    Ok(ApiResponse::created(keyword))
}

/// POST /api/v1/keywords/{id}/update — Update a keyword
///
/// All fields optional — only provided fields are updated.
/// Returns HTTP 200 with the updated Keyword, or 404 if not found.
pub async fn update_keyword(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateKeywordRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if let Some(ref word) = req.word {
        if word.trim().is_empty() {
            return Err(AppError::BadRequest("word must not be empty".into()));
        }
    }
    if let Some(sm) = req.std_multiplier {
        if sm <= 0.0 {
            return Err(AppError::BadRequest(
                "std_multiplier must be positive".into(),
            ));
        }
    }
    if let Some(mhc) = req.min_hot_count {
        if mhc <= 0 {
            return Err(AppError::BadRequest("min_hot_count must be >= 1".into()));
        }
    }

    // Verify the keyword exists
    let _existing = db::keyword::get_keyword_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Keyword {} not found", id)))?;

    let updated = db::keyword::update_keyword(&state.pool, id, &req)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Keyword {} not found", id)))?;

    Ok(ApiResponse::ok(updated))
}

/// POST /api/v1/keywords/{id}/delete — Delete a keyword
///
/// Returns HTTP 204 on success, or 404 if the keyword does not exist.
pub async fn delete_keyword(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Verify existence first
    let exists = db::keyword::get_keyword_by_id(&state.pool, id).await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("Keyword {} not found", id)));
    }

    db::keyword::delete_keyword(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
