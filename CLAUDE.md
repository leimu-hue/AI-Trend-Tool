# CLAUDE.md

Guidance for Claude Code working with this repo.

## Project Overview

**TrendAITool** – AI hotspot monitoring system.  
Users manage RSS data sources and keywords via React web UI. System auto-fetches content, matches keywords (Aho-Corasick), detects trending hotspots (statistical burst detection: moving average + standard deviation over hourly buckets), pushes alerts through DingTalk/Feishu webhooks.

## Architecture – Event-Driven Pipeline

Three background modules connected via `tokio::mpsc` event channels, with timer fallback:

1. **Parser** – Polls due sources every `parser.interval_seconds` (default 30s). Fetches RSS/Atom feeds, writes new articles to `articles` (deduped by `link`). Sends `articles_ready` event to Filter when new articles are inserted.
2. **Filter** – Triggered by timer (`filter.interval_seconds`, default 300s) **or** `articles_ready` event from Parser.
   - Reads unprocessed articles (`processed_at IS NULL`), matches keywords (Aho-Corasick, case-sensitive & case-insensitive).
   - Records matches in `keyword_mentions`, accumulates hourly bucket counts.
   - Detects hotspots when `current_count > mean + (std_multiplier × stddev)` and `count >= min_hot_count` (past 24 hours).
   - Upserts `hot_events` (unique on `keyword_id + hour_bucket`) and creates `push_records` (status=pending).
   - Sends `push_ready` event to Pusher when new push records are created.
3. **Pusher** – Triggered by timer (`pusher.interval_seconds`, default 10s) **or** `push_ready` event from Filter.
   - Polls `push_records` with status `pending` or retry-due.
   - Sends webhook POST, updates status. Linear backoff (`retry_count × retry_base_seconds`, max 3 retries).
   - Optimistic locking prevents duplicate sends.

All modules run together in a single process. Graceful shutdown via `CancellationToken` (Ctrl+C).
Manual trigger endpoints: `POST /api/v1/trigger/filter` and `POST /api/v1/trigger/pusher`.

## Key Database Tables

- `api_tokens` – Bearer tokens (revocable, optional expiry)
- `data_sources` – RSS feed configs (URL, interval, JSON config)
- `articles` – Fetched articles, `processed_at` tracks filter state
- `keywords` – Keywords with sensitivity params (`std_multiplier`, `min_hot_count`, `case_sensitive`)
- `keyword_mentions` – Per-article keyword match records (keyword_id + article_id)
- `hot_events` – Detected hotspots (hourly bucket stats, unique on `keyword_id + hour_bucket` for upsert)
- `push_channels` – Alert channel configs (webhook URL in JSON)
- `push_records` – Per-hotspot per-channel status & retry tracking (unique: `hot_event_id, channel_id`)

## Commands

```bash
# Run all modules (API + Parser + Filter + Pusher) in one process
# Config path is the first positional argument (defaults to "config.toml")
cargo run -- config.toml

# Database migrations (auto-run on startup via sqlx::migrate!)
cargo sqlx migrate run

# Frontend (Electron + React)
cd web && npm run dev      # dev server
cd web && npm run build    # production build

# Production backend build
cargo build --release
```

## Development Rules

### SQL Organization
- **All SQL queries MUST live in `src/db/<module>.rs`** (e.g., `db/token.rs`, `db/article.rs`).
- `handlers/`, `middleware/`, `services/`, `routes.rs`, and `main.rs` MUST NOT contain raw `sqlx::query*` or inline SQL strings.
- Every database operation must be a named function in the corresponding `db/` module, accepting `&SqlitePool` as the first parameter.
- Handlers/middleware call `db::<module>::<function>()` — never execute SQL directly.

### HTTP Methods
- **Only use `GET` and `POST`** for all API endpoints.
- Do NOT use `DELETE`, `PUT`, or `PATCH`.
- Semantics expressed via URL path:
  - Create → `POST /resource`
  - Read/List → `GET /resource`
  - Update → `POST /resource/update/{id}` or `POST /resource/{id}/update`
  - Delete/Revoke → `POST /resource/delete/{id}` or `POST /resource/revoke/{id}`

### Middleware Pattern
- Use `middleware::from_fn_with_state(state, auth_middleware)` for authenticated routes.
- Auth middleware lives in `src/middleware/auth.rs` and extracts `State<AppState>`.
- `AppState` (defined in `routes.rs`) carries `SqlitePool`, `AppConfig`, and `Pipeline`.

### API Documentation
- API docs live in `docs/apis/` as Markdown files (e.g., `token-api.md`).
- When adding, modifying, or removing an endpoint, **always update the corresponding `docs/apis/*.md`** to keep URL, method, params, request/response schema, and examples in sync.
- Each MD file groups endpoints by domain (token, source, keyword, etc.).

### Pipeline Module
- `src/pipeline.rs` defines the `Pipeline` struct: inter-module `mpsc` channels (`articles_ready_tx`, `push_ready_tx`) + shared `CancellationToken`.
- `Pipeline::new()` returns `(Pipeline, articles_rx, push_rx)` — main.rs distributes receivers to downstream modules.
- All background loops use `tokio::select!` to listen for cancellation alongside timer/event triggers.