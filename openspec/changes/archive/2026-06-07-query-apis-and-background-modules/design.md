## Context

TrendAITool pipeline has three stages: Parser → Filter → Pusher. Steps 01-04 built project scaffolding, data models, auth, and CRUD APIs. The `src/services.rs` placeholder exists, config structs for Parser/Filter/Pusher are defined, dependencies (`feed-rs`, `aho-corasick`, `reqwest`) are declared. The system cannot yet fetch RSS, detect hotspots, or push alerts. Query APIs for articles, hotspots, and trends are missing.

**Constraints:**
- All SQL must live in `src/db/<module>.rs` (project rule)
- Only `GET` and `POST` HTTP methods allowed (project rule)
- Module convention: `src/X.rs` + `src/X/` directory, NO `mod.rs` (spec: `module-organization`)
- Database: SQLite via `sqlx 0.7`, WAL mode, foreign keys enforced
- Auth: Bearer token middleware on all `/api/v1/` routes

## Goals / Non-Goals

**Goals:**
- 4 query endpoints (articles, hotspots, push-records, trend) following project HTTP conventions
- 2 trigger endpoints (filter, pusher) for manual invocation
- Parser module: periodic RSS fetch, concurrency-limited, link-deduplicated
- Filter module: Aho-Corasick multi-pattern matching, hourly bucket counting, statistical burst detection
- Pusher module: webhook dispatch, exponential backoff retry, optimistic locking
- Background services run on `tokio::spawn` with mode-based gating (`all` | `api` | `parser` | `filter` | `pusher`)

**Non-Goals:**
- RSS feed types beyond RSS 2.0/Atom (only `feed-rs` parser for now)
- Push channels beyond webhook (DingTalk/Feishu format is a webhook POST)
- Real-time streaming (polling-based pipeline is sufficient)
- Parser auto-discovery (source type is explicitly `rss`)

## Decisions

### D1: Trend API uses `hot_events` table (not `keyword_mentions` JOIN)

**Choice:** Query `hot_events.hour_bucket` + `hot_events.count` directly, grouped per keyword.

**Rationale:** `hot_events` already stores pre-aggregated hourly counts from the Filter module. Querying `keyword_mentions` JOIN `articles` with `strftime` GROUP BY would be redundant, slower, and inconsistent with what Filter produces. Using `hot_events` keeps the trend API consistent with hotspot detection output.

**Alternative considered:** Design doc proposed `keyword_mentions` JOIN approach. Rejected for consistency and performance.

### D2: `src/services.rs` module entry + `src/services/` subdirectory (no `mod.rs`)

**Choice:** `src/services.rs` declares `pub mod parser; pub mod filter; pub mod pusher;`, with implementations in `src/services/parser.rs`, `src/services/filter.rs`, `src/services/pusher.rs`.

**Rationale:** Project spec `module-organization` prohibits `mod.rs`. The design doc incorrectly used `src/services/mod.rs` — corrected to follow the 2018 edition convention already used by `models/`, `handlers/`, `middleware/`, `db/`.

### D3: Query SQL lives in `db/` modules

**Choice:** New query functions (`count_articles`, `list_hotspots_paginated`, `count_hotspots`, `get_trend_points`, `list_pushable_records`) added to existing `db/article.rs`, `db/hot_event.rs`, `db/push_record.rs`.

**Rationale:** Project rule: handlers never contain raw SQL. handlers call db functions, pass `&SqlitePool`.

**Alternative considered:** Design doc showed inline SQL in handlers. Rejected per project convention.

### D4: Parser trait with `async-trait` for extensibility

**Choice:** Define `Parser` trait using `async_trait` macro, implement `RssParser`. Add `async-trait` to `Cargo.toml`.

**Rationale:** Design doc's trait-based design enables adding new parser types (JSON Feed, Atom, etc.) without changing the scheduler loop. `async-trait` is the standard crate for async trait methods in Rust. Worth the dependency cost now to avoid refactoring later.

### D5: Shared `run_*_once` functions for loop + manual trigger

**Choice:** Filter and Pusher expose `pub async fn run_filter_once(pool, config)` and `pub async fn run_pusher_once(pool, config)`. Background loops call these on tick; trigger handlers call them directly.

**Rationale:** Avoids code duplication. The design doc already structures code this way — confirmed.

### D6: Mode-based background task gating

**Choice:** `main.rs` checks `cli.mode` at startup; `all` and `api` spawn all three background tasks; standalone modes (`parser`/`filter`/`pusher`) spawn only the relevant task and await ctrl-c.

**Rationale:** Matches the design doc's approach. Keeps the CLI simple: `cargo run -- all` for full system, `cargo run -- api` for server-only, individual modes for debugging.

## Risks / Trade-offs

- **SQLite concurrent writes:** Parser+Filter+Pusher write concurrently. SQLite WAL mode handles this, but `SQLITE_BUSY` is possible under heavy load. → Mitigation: pool max_connections=5, WAL mode enabled, retry logic in Pusher.
- **`async-trait` heap allocation:** Each async trait call boxes the future. → Acceptable: Parser loop runs every 30s with low call volume; overhead negligible.
- **Filter batch boundary:** If `batch_size` (default 100) splits articles, same-hour keyword counts may be incomplete until next Filter tick. → Mitigation: Filter runs every 5 min; this is acceptable for a near-real-time system with hourly bucket granularity.
- **No hot-event dedup across Filter runs:** If Filter runs twice within the same hour, it may create duplicate `hot_events` for the same `(keyword_id, hour_bucket)`. → Mitigation: Use `INSERT OR REPLACE` (or upsert) on `(keyword_id, hour_bucket)` to make hourly counts idempotent. This needs a UNIQUE constraint — current schema does not have one. Will add via migration or use `ON CONFLICT` upsert pattern.

## Open Questions

None — all design ambiguities resolved during proposal phase.
