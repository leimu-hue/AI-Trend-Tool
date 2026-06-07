## 1. Project initialization

- [x] 1.1 Run `cargo init --name trend-monitor` in project root
- [x] 1.2 Create `Cargo.toml` with all dependencies: axum, tower, tokio, sqlx (SQLite), serde, toml, chrono, tracing, feed-rs, aho-corasick, reqwest, rand, hex, clap
- [x] 1.3 Create `config.toml` with [server], [database], [auth], [parser], [filter], [pusher] sections. `[auth].initial_token` is optional

## 2. Core source files

- [x] 2.1 Create `src/config.rs` — `AppConfig` and sub-structs with `Deserialize`, `AppConfig::load(path)` method
- [x] 2.2 Create `src/error.rs` — `AppError` enum (NotFound, BadRequest, Unauthorized, Conflict, Internal, Database) with `IntoResponse` impl and `From<sqlx::Error>` impl, plus `ApiResponse` helper
- [x] 2.3 Create `src/db.rs` — `init_pool()` function, SQLite pool with WAL mode and foreign keys ON
- [x] 2.4 Create `src/routes.rs` — `create_router()` with `/health` (returns `{"status": "ok"}`), `/api/v1` nest, CORS layer, `AppState` struct

## 3. Entry point

- [x] 3.1 Create `src/main.rs` — clap CLI (--config flag + positional mode), config load, DB init, migration run, axum serve
- [x] 3.2 Verify `cargo check` passes with no errors
- [x] 3.3 Verify `cargo run -- --config config.toml all` starts and `curl /health` returns `{"status": "ok"}`

## 4. Module structure (modern Rust style, no mod.rs)

- [x] 4.1 Create `src/models.rs` (module entry, empty or with placeholder `pub mod` comments for step 02)
- [x] 4.2 Create empty `src/models/` directory
- [x] 4.3 Create `src/handlers.rs` (module entry, empty or with placeholder `pub mod` comments for step 03)
- [x] 4.4 Create empty `src/handlers/` directory
- [x] 4.5 Create `src/middleware.rs` (module entry, empty or with placeholder `pub mod` comments for step 03)
- [x] 4.6 Create empty `src/middleware/` directory
- [x] 4.7 Create `src/services.rs` (module entry, empty or with placeholder `pub mod` comments for step 05)
- [x] 4.8 Create empty `src/services/` directory
- [x] 4.9 Create empty `docs/migrations/` directory (for step 02)
- [x] 4.10 Confirm no `mod.rs` files exist under `src/`

## 5. Final verification

- [x] 5.1 Run `cargo build` — zero errors
- [x] 5.2 Run `cargo run -- --config config.toml all` — server starts, listens on configured host:port
- [x] 5.3 Run `curl http://localhost:8080/health` — returns HTTP 200 with `{"status": "ok"}`
- [x] 5.4 Review: all files in expected file list exist (see plan section "预期文件清单")
