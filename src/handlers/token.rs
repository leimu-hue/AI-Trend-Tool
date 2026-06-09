use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;

use crate::db;
use crate::error::{ApiResponse, AppError};
use crate::models::token::{ApiToken, ApiTokenInfo, CreateTokenRequest};
use crate::routes::AppState;

/// POST /api/v1/tokens — Create a new API token
///
/// Generates a 64-character random hex token, inserts into api_tokens,
/// and returns the full ApiToken including the plaintext token value.
/// The plaintext token is only returned once at creation time.
pub async fn create_token(
    State(state): State<AppState>,
    Json(req): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name must not be empty".into()));
    }

    // Generate 64-character random hex token (32 random bytes)
    let bytes: [u8; 32] = rand::thread_rng().gen();
    let token_str = hex::encode(bytes);

    let token: ApiToken =
        db::token::create_token(&state.pool, &req.name, &token_str, req.expires_at).await?;

    Ok(ApiResponse::created(token))
}

/// GET /api/v1/tokens — List all tokens (hides plaintext token values)
///
/// Returns ApiTokenInfo for each token, which excludes the `token` field.
/// Ordered by created_at descending (newest first).
pub async fn list_tokens(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tokens: Vec<ApiToken> = db::token::list_tokens(&state.pool).await?;

    let infos: Vec<ApiTokenInfo> = tokens.into_iter().map(ApiTokenInfo::from).collect();
    Ok(ApiResponse::ok(infos).1)
}

/// POST /api/v1/tokens/revoke/{id} — Revoke a token (soft delete)
///
/// Sets revoked = 1 for the given token ID.
/// Returns 204 on success or 404 if the token does not exist.
pub async fn revoke_token(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Check existence first
    let exists = db::token::get_token_by_id(&state.pool, id).await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!(
            "Token with id {} not found",
            id
        )));
    }

    db::token::revoke_token(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
