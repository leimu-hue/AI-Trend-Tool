## Context

Greenfield Rust backend for TrendAITool — an AI hotspot monitoring system. No existing code. This design establishes the foundational architecture that all subsequent steps build on.

The plan (`docs/plans/01-backend-project-setup.md`) defines the exact file structure, dependency versions, and code patterns. The design rationale below explains the architectural choices behind those decisions.

## Goals / Non-Goals

**Goals:**
- Runnable Rust project with `cargo build` from day one
- Typed configuration via TOML — one file, all settings, parse-on-startup
- SQLite connection pool with WAL mode for concurrent read/write
- Unified error response format across all future API endpoints
- Health check endpoint responding to HTTP GET
- Modern Rust module organization (2018 edition style, no `mod.rs`)

**Non-Goals:**
- Database migrations (step 02)
- Authentication, CRUD APIs, background modules (steps 03-05)
- Frontend (separate plan)
- Docker/CI/CD setup
- Tests (added incrementally in later steps)

## Decisions

### 1. SQLite as database

**Choice**: SQLite via `sqlx` with `runtime-tokio` and `chrono` features.

**Rationale**: Single-file database, zero setup, no external service. Fits the tool's scale — a monitoring system typically runs on a single machine, not a distributed cluster. WAL mode enables concurrent reads during writes, sufficient for the expected throughput (background modules + API). SQLite's 2GB+ capacity exceeds the lifetime article volume.

**Alternatives considered**:
- PostgreSQL: Overkill at this stage. Requires external service. Add complexity without benefit for single-machine deployment.
- rusqlite: Synchronous API incompatible with async axum handlers. sqlx provides async pool natively.

### 2. axum web framework

**Choice**: axum 0.7 with tower/tower-http ecosystem.

**Rationale**: Built on tokio + tower + hyper — the Rust async standard stack. Type-safe extractors (State, Path, Query, Json) eliminate boilerplate parsing. Tower middleware layer (CORS, tracing, auth) composes cleanly. Active community, good docs.

**Alternatives considered**:
- actix-web: Mature but uses its own runtime. Heavier abstraction. Less aligned with tower ecosystem.
- warp: Filter-based composition gets unwieldy with many routes. axum's `Router::nest` is cleaner for `/api/v1` nesting.
- poem: Good design but smaller community, fewer examples.

### 3. Modern module style (no mod.rs)

**Choice**: `src/models.rs` + `src/models/token.rs` pattern. No `mod.rs` files.

**Rationale**: Rust 2018 edition recommendation. Each module's entry file name matches what you `mod` — `src/models.rs` is the `models` module. Editor tabs show filenames not a dozen `mod.rs`. Already adopted by tokio, axum, serde.

**Alternatives considered**:
- Classic `src/models/mod.rs`: Works but produces many indistinguishable `mod.rs` tabs. Less discoverable in file tree.

### 4. Unified error handling

**Choice**: `AppError` enum mapping errors to HTTP status codes + JSON body `{"error": {"code": "...", "message": "..."}}`.

**Rationale**: Single `impl IntoResponse for AppError` guarantees consistent error format. `From<sqlx::Error>` impl makes `?` just work in handlers — database errors auto-convert. Each variant maps to a semantic `code` string (`NOT_FOUND`, `UNAUTHORIZED`) for frontend parsing.

**Alternatives considered**:
- thiserror + separate response mapping: More boilerplate, two-step error handling.
- anyhow throughout: Loses structured error semantics, harder to return correct HTTP status codes.

### 5. TOML config with typed deserialization

**Choice**: `config.toml` parsed into `AppConfig` struct via `serde::Deserialize`. No env vars, no CLI overrides for config values.

**Rationale**: One file, one truth. TOML is human-readable and supports sections natively. Typed structs catch config errors at startup (wrong type = panic with clear message), not at runtime.

**Alternatives considered**:
- figment/config crate: More flexible (env vars, multi-source) but heavier dependency. Unnecessary for single-file config.
- Environment variables: Work for deployment but painful for local dev. Harder to document all settings in one place.
- YAML: Requires explicit serde_yaml dep. TOML is more Rust-idiomatic (used by Cargo itself).

### 6. CLI via clap

**Choice**: clap 4 with derive macros. Two arguments: `--config` (default `config.toml`) and `mode` (positional, default `all`).

**Rationale**: Same binary serves different run modes (`all`, `api`, `parser`, `filter`, `pusher`). clap provides help text, validation, and shell completions for free.

## Risks / Trade-offs

- **SQLite concurrency under heavy webhook bursts**: WAL mode handles moderate concurrent access. If push_records polling at 10s intervals contends with API writes, could manifest as `SQLITE_BUSY`. Mitigation: connection pool with 5 connections; increase if profiling shows contention.
- **Single config file, no hot-reload**: Config changes require restart. Acceptable for self-hosted monitoring tool. Hot-reload adds complexity (file watcher, `Arc<RwLock<AppConfig>>`) for minimal benefit.
- **No mod.rs rule is a convention, not enforced by compiler**: New contributors might create `mod.rs`. Mitigation: document in CLAUDE.md, catch in code review.
- **Rust compile times**: First `cargo build` downloads and compiles all dependencies (~2-5 min). Acceptable one-time cost. CI should cache `target/`.

## Open Questions

1. ~~Should the initial_token in `[auth]` section be required or optional?~~ **Resolved: optional.** `Option<String>` as plan specifies. If not set, admin creates token via DB directly for first access.
2. ~~Should the health check endpoint return structured JSON or plain text?~~ **Resolved: structured JSON.** Return `{"status": "ok"}` instead of plain text. Note: this deviates from plan which used `"ok"`.
