use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::handlers::{channel, keyword, query, source, token};
use crate::middleware::auth::auth_middleware;
use crate::pipeline::Pipeline;

pub fn create_router(pool: SqlitePool, config: AppConfig, pipeline: Pipeline) -> Router {
    let state = AppState {
        pool: pool.clone(),
        config,
        pipeline,
    };

    // ── API routes ──
    let api = Router::new()
        .route("/tokens", post(token::create_token))
        .route("/tokens", get(token::list_tokens))
        .route("/tokens/revoke/{id}", post(token::revoke_token))
        // Sources API
        .route("/sources", get(source::list_sources))
        .route("/sources", post(source::create_source))
        .route("/sources/{id}/update", post(source::update_source))
        .route("/sources/{id}/delete", post(source::delete_source))
        .route("/sources/{id}/fetch", post(source::trigger_fetch))
        // Keywords API
        .route("/keywords", get(keyword::list_keywords))
        .route("/keywords", post(keyword::create_keyword))
        .route("/keywords/{id}/update", post(keyword::update_keyword))
        .route("/keywords/{id}/delete", post(keyword::delete_keyword))
        // Channels API
        .route("/channels", get(channel::list_channels))
        .route("/channels", post(channel::create_channel))
        .route("/channels/{id}/update", post(channel::update_channel))
        .route("/channels/{id}/delete", post(channel::delete_channel))
        // Query API
        .route("/articles", get(query::list_articles))
        .route("/hotspots", get(query::list_hotspots))
        .route("/hotspots/{id}/push-records", get(query::get_push_records))
        .route("/trend/{keyword_id}", get(query::get_trend))
        // System control
        .route("/trigger/filter", post(query::trigger_filter))
        .route("/trigger/pusher", post(query::trigger_pusher))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

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
    pub pipeline: Pipeline,
}
