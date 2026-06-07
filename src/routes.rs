use axum::{routing::get, Json, Router};
use serde_json::json;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;

pub fn create_router(pool: SqlitePool, config: AppConfig) -> Router {
    let state = AppState { pool, config };

    Router::new()
        // Health check (no auth required)
        .route("/health", get(health_check))
        // API v1 (auth middleware added in step 03)
        .nest("/api/v1", api_routes())
        .with_state(state)
        .layer(CorsLayer::permissive())
}

fn api_routes() -> Router<AppState> {
    Router::new()
    // Token API (step 03)
    // Sources API (step 04)
    // Keywords API (step 04)
    // Channels API (step 04)
    // Query API (step 05)
    // System control (step 05)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
}
