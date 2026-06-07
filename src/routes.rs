use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::handlers::token;
use crate::middleware::auth::auth_middleware;

pub fn create_router(pool: SqlitePool, config: AppConfig) -> Router {
    let state = AppState {
        pool: pool.clone(),
        config,
    };

    // ── API routes ──
    let api = Router::new()
        .route("/tokens", post(token::create_token))
        .route("/tokens", get(token::list_tokens))
        .route("/tokens/revoke/{id}", post(token::revoke_token))
        // Sources API (step 04)
        // Keywords API (step 04)
        // Channels API (step 04)
        // Query API (step 05)
        // System control (step 05)
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api)
        .layer(CorsLayer::permissive())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
}
