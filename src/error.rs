use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    /// 404 - Resource not found
    NotFound(String),
    /// 400 - Bad request
    #[allow(dead_code)]
    BadRequest(String),
    /// 401 - Unauthorized
    Unauthorized(String),
    /// 409 - Conflict (e.g., unique constraint violation)
    Conflict(String),
    /// 500 - Internal error
    #[allow(dead_code)]
    Internal(String),
    /// 500 - Database error (auto-converted from sqlx::Error)
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Internal server error".to_string(),
                )
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::BadRequest(errors.to_string())
    }
}

pub struct ApiResponse;

impl ApiResponse {
    /// 200 OK + JSON body
    pub fn ok<T: serde::Serialize>(data: T) -> (StatusCode, Json<serde_json::Value>) {
        (StatusCode::OK, Json(json!({ "data": data })))
    }

    /// 201 Created + JSON body
    pub fn created<T: serde::Serialize>(data: T) -> (StatusCode, Json<serde_json::Value>) {
        (StatusCode::CREATED, Json(json!({ "data": data })))
    }

    /// 204 No Content
    #[allow(dead_code)]
    pub fn no_content() -> StatusCode {
        StatusCode::NO_CONTENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn not_found_returns_404() {
        let response = AppError::NotFound("test".into()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn database_error_returns_500() {
        let response = AppError::Database(sqlx::Error::PoolTimedOut).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
