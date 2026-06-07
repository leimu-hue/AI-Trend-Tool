use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use chrono::Utc;

use crate::db;
use crate::error::AppError;
use crate::models::token::ApiToken;
use crate::routes::AppState;

/// Bearer Token authentication middleware.
/// Extracts token from Authorization header, validates against database,
/// checks revocation/expiry, updates last_used_at in background, and
/// injects ApiToken into request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Extract Bearer token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token_str = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid Authorization format, expected Bearer".to_string())
        })?;

    // 2. Query database for valid (non-revoked) token
    let token: ApiToken = db::token::get_token_by_value(&state.pool, token_str)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid or revoked token".to_string()))?;

    // 3. Check expiry
    if let Some(expires_at) = token.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(AppError::Unauthorized("Token has expired".to_string()));
        }
    }

    // 4. Update last_used_at in background (fire-and-forget)
    let pool = state.pool.clone();
    let token_id = token.id;
    tokio::spawn(async move {
        let _ = db::token::update_token_last_used(&pool, token_id).await;
    });

    // 5. Inject ApiToken into request extensions for downstream handlers
    request.extensions_mut().insert(token);

    Ok(next.run(request).await)
}
