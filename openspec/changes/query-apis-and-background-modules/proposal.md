## Why

The project currently has CRUD APIs for data sources, keywords, and channels, but lacks read APIs to surface fetched articles, detected hotspots, push records, and trend data. The three core background modules (Parser, Filter, Pusher) that form the pipeline are not yet implemented, so the system cannot fetch RSS feeds, match keywords, detect hotspots, or push alerts. This change delivers the query surface and the pipeline that together make TrendAITool a functioning hotspot monitoring system.

## What Changes

- Add 4 data query endpoints: article list (paginated + filterable), hotspot list, push records by hotspot, and keyword trend curve
- Add 2 system control endpoints: manual trigger for filter and pusher
- Implement Parser background module: RSS feed fetching via `feed-rs`, configurable concurrency, periodic scheduling, deduplication by link
- Implement Filter background module: Aho-Corasick keyword matching, hourly bucket counting, statistical burst detection (moving average + stddev), `hot_events` creation, `push_records` generation
- Implement Pusher background module: webhook POST to push channels, exponential backoff retry (max 3), optimistic locking
- Wire background modules into `main.rs` with mode-based launch (`all`, `api`, `parser`, `filter`, `pusher`)
- Add `async-trait` dependency to `Cargo.toml` for the extensible Parser trait
- All new SQL queries live in `src/db/<module>.rs` per project convention

## Capabilities

### New Capabilities

- `query-apis`: Article list, hotspot list, push records, and keyword trend data endpoints
- `trigger-apis`: Manual trigger endpoints for running filter and pusher on demand
- `parser-module`: RSS feed parsing background service with configurable scheduling
- `filter-module`: Keyword matching + statistical hotspot detection background service
- `pusher-module`: Webhook push + retry background service with exponential backoff

### Modified Capabilities

- `module-organization`: Services module converts from placeholder `src/services.rs` to directory-backed module (`src/services.rs` + `src/services/` directory with submodules)

## Impact

- **Files created**: `src/services/parser.rs`, `src/services/filter.rs`, `src/services/pusher.rs`, `src/handlers/query.rs`
- **Files modified**: `src/services.rs`, `src/handlers.rs`, `src/routes.rs`, `src/main.rs`, `Cargo.toml`
- **DB modules extended**: `src/db/article.rs`, `src/db/hot_event.rs`, `src/db/push_record.rs` (new query functions)
- **Dependencies added**: `async-trait`
- **No breaking changes**: All existing APIs unchanged, new endpoints are additive
