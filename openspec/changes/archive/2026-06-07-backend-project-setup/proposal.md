## Why

TrendAITool needs a compilable Rust backend skeleton as the foundation for all subsequent work. Without it, no database migrations, no API endpoints, no background modules can be built. This is the first step — establish the project structure, dependencies, configuration system, error handling, and a running HTTP server that passes a health check.

## What Changes

- Initialize Rust project `trend-monitor` with full dependency manifest in `Cargo.toml`
- Create `config.toml` with server, database, auth, parser, filter, and pusher sections
- Implement `src/config.rs` — typed config structs with TOML deserialization
- Implement `src/error.rs` — unified `AppError` enum covering 404/400/401/409/500 with `IntoResponse` impl and `From<sqlx::Error>` conversion, plus `ApiResponse` helper for 200/201/204 success responses
- Implement `src/db.rs` — SQLite connection pool init with WAL mode and foreign keys enabled
- Implement `src/main.rs` — CLI argument parsing (clap), config loading, DB init, migration run, axum server startup
- Implement `src/routes.rs` — router skeleton with `/health` endpoint, `/api/v1` nest, CORS layer, and `AppState` struct
- Create empty module entry files (`models.rs`, `handlers.rs`, `middleware.rs`, `services.rs`) with corresponding submodule directories for later steps
- Create empty `docs/migrations/` directory for step 02
- **All modules use Rust 2018 edition "non-mod.rs" style** — `src/models.rs` + `src/models/` (no `mod.rs` files anywhere)

## Capabilities

### New Capabilities

- `backend-project-scaffold`: Rust project skeleton with Cargo.toml, config parsing, axum HTTP server, unified error handling, SQLite connection pool, and CLI entry point
- `module-organization`: Modern Rust module style (2018 edition) — `module.rs` + `module/` directory pattern, no `mod.rs` files

### Modified Capabilities

<!-- None — this is the first change -->

## Impact

- **New files**: `Cargo.toml`, `config.toml`, `src/main.rs`, `src/config.rs`, `src/error.rs`, `src/db.rs`, `src/routes.rs`, `src/models.rs`, `src/handlers.rs`, `src/middleware.rs`, `src/services.rs`, plus empty directories `src/models/`, `src/handlers/`, `src/middleware/`, `src/services/`, `docs/migrations/`
- **Dependencies**: axum 0.7, tower 0.4, tower-http 0.5, tokio 1, sqlx 0.7 (SQLite), serde/toml, chrono, tracing, feed-rs, aho-corasick, reqwest, rand/hex, clap
- **No breaking changes** — this is a greenfield project
- **Verification**: `cargo check` passes, `cargo run` starts server, `curl /health` returns "ok"
