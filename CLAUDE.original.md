# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

**TrendAITool** – AI hotspot monitoring system.  
Users manage RSS data sources and keywords via a React web UI. The system automatically fetches content, matches keywords (Aho-Corasick), detects trending hotspots (statistical burst detection: moving average + standard deviation over hourly buckets), and pushes alerts through DingTalk/Feishu webhooks.

## Architecture – Pipeline Pattern

Three independent background modules run on separate schedules:

1. **Parser** – Fetches RSS feeds at configured intervals, writes new articles to `articles` (deduped by `link`).
2. **Filter** – Runs every 5 min.  
   - Reads unprocessed articles (`processed_at IS NULL`), matches keywords (Aho-Corasick).  
   - Accumulates hourly bucket counts.  
   - Detects hotspots when `current_count > mean + (std_multiplier × stddev)` and `count >= min_hot_count` (over past 24 hours).  
   - Creates `hot_events` and `push_records` (status=pending).
3. **Pusher** – Runs every 10 sec.  
   - Polls `push_records` with status `pending` or retry-due.  
   - Sends webhook POST, updates status. Exponential backoff (max 3 retries).  
   - Optimistic locking prevents duplicate sends.

All modules can run together (`hotspot all`) or independently (`hotspot api|parser|filter|pusher`).

## Key Database Tables

- `api_tokens` – Bearer tokens (revocable, optional expiry)
- `data_sources` – RSS feed configs (URL, interval, JSON config)
- `articles` – Fetched articles, `processed_at` tracks filter state
- `keywords` – Keywords with sensitivity params (`std_multiplier`, `min_hot_count`)
- `hot_events` – Detected hotspots (hourly bucket stats)
- `push_channels` – Alert channel configs (webhook URL in JSON)
- `push_records` – Per-hotspot per-channel status & retry tracking (unique: `hot_event_id, channel_id`)

## Commands

```bash
# Run all modules in one process
cargo run -- --config config.toml all

# Run individual modules
cargo run -- api       # API server only
cargo run -- parser    # Parser only
cargo run -- filter    # Filter only
cargo run -- pusher    # Pusher only

# Database migrations
cargo sqlx migrate run

# Frontend
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
- Semantics are expressed via URL path:
  - Create → `POST /resource`
  - Read/List → `GET /resource`
  - Update → `POST /resource/update/{id}` or `POST /resource/{id}/update`
  - Delete/Revoke → `POST /resource/delete/{id}` or `POST /resource/revoke/{id}`

### Middleware Pattern
- Use `middleware::from_fn_with_state(state, auth_middleware)` for authenticated routes.
- Auth middleware lives in `src/middleware/auth.rs` and extracts `State<AppState>`.
- The axum 0.7.9 `Path<T>` + `from_fn` routing bug is resolved in axum 0.8+ — `Path` extractors work correctly with `from_fn_with_state` middleware.

### API Documentation
- API docs live in `docs/apis/` as Markdown files (e.g., `token-api.md`).
- When adding, modifying, or removing an endpoint, **always update the corresponding `docs/apis/*.md`** to keep URL, method, params, request/response schema, and examples in sync.
- Each MD file groups endpoints by domain (token, source, keyword, etc.).