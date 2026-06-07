## 1. Dependencies and module structure

- [x] 1.1 Add `async-trait` to `Cargo.toml` dependencies
- [x] 1.2 Create `src/services/` directory
- [x] 1.3 Update `src/services.rs` from placeholder to `pub mod parser; pub mod filter; pub mod pusher;`
- [x] 1.4 Add `pub mod query;` to `src/handlers.rs`
- [x] 1.5 Add `pub mod keyword_mention;` to `src/models.rs`

## 2. DB layer — new query functions

- [x] 2.1 Fix `count_articles` in `src/db/article.rs` to bind filter parameters (currently missing param bindings)
- [x] 2.2 Add `mark_processed_batch` to `src/db/article.rs` for bulk updating `processed_at` on article IDs
- [x] 2.3 Add `list_hotspots_paginated` and `count_hotspots` to `src/db/hot_event.rs` with optional `keyword_id` filter, LIMIT, OFFSET
- [x] 2.4 Add `list_push_records_with_details` to `src/db/push_record.rs` — combined query joining push_records with push_channels for richer push record info (or reuse existing `get_push_records_by_hot_event`)
- [x] 2.5 Create `src/db/keyword_mention.rs` with `insert_keyword_mention(keyword_id, article_id)`
- [x] 2.6 Register `pub mod keyword_mention;` in `src/db.rs`
- [x] 2.7 Create `src/models/keyword_mention.rs` with `KeywordMention` struct (id, keyword_id, article_id, matched_at)

## 3. Query API handlers

- [x] 3.1 Create `src/handlers/query.rs` with `PaginatedResponse<T>` generic struct
- [x] 3.2 Implement `list_articles` handler — calls `db::article::list_articles` + `db::article::count_articles`, returns paginated response
- [x] 3.3 Implement `list_hotspots` handler — calls new `db::hot_event::list_hotspots_paginated` + `count_hotspots`, supports `keyword_id` filter
- [x] 3.4 Implement `get_push_records` handler — calls `db::push_record::get_push_records_by_hot_event`, returns push records for a hotspot
- [x] 3.5 Implement `get_trend` handler — calls `db::hot_event::get_hourly_counts` + `db::keyword::get_keyword_by_id`, returns `TrendResponse` with points

## 4. Trigger API handlers

- [x] 4.1 Implement `trigger_filter` handler — calls `crate::services::filter::run_filter_once`
- [x] 4.2 Implement `trigger_pusher` handler — calls `crate::services::pusher::run_pusher_once`

## 5. Route registration

- [x] 5.1 Register query routes in `src/routes.rs`: `/articles` (GET), `/hotspots` (GET), `/hotspots/{id}/push-records` (GET), `/trend/{keyword_id}` (GET)
- [x] 5.2 Register trigger routes in `src/routes.rs`: `/trigger/filter` (POST), `/trigger/pusher` (POST)
- [x] 5.3 Add `use crate::handlers::query;` import in `src/routes.rs`

## 6. Parser module

- [x] 6.1 Create `src/services/parser.rs` with `ParsedArticle` struct and `Parser` trait (using `#[async_trait]`)
- [x] 6.2 Implement `RssParser` struct with `new(config: &ParserConfig)` constructor and `fetch_and_parse` method
- [x] 6.3 Implement `start_parser_loop` — 30-second ticker, queries due sources, spawns fetch tasks with semaphore concurrency control, calls `db::article::insert_article` + `db::source::update_source_last_fetched`

## 7. Filter module

- [x] 7.1 Create `src/services/filter.rs` with `run_filter_once(pool, config)` — shared by background loop and manual trigger
- [x] 7.2 Implement unprocessed article loading — calls `db::article::get_unprocessed_articles`
- [x] 7.3 Implement Aho-Corasick automaton build from enabled keywords (`db::keyword::list_enabled_keywords`)
- [x] 7.4 Implement keyword matching loop — iterate articles, run AC find_iter, call `db::keyword_mention::insert_keyword_mention`, accumulate hourly counts
- [x] 7.5 Implement burst detection — load historical counts from `db::hot_event::get_hourly_counts`, compute mean/stddev, compare against threshold, create `hot_events` and `push_records` when hotspot detected
- [x] 7.6 Implement article batch marking — call `db::article::mark_processed_batch`
- [x] 7.7 Implement `start_filter_loop` — configurable interval ticker, calls `run_filter_once`

## 8. Pusher module

- [x] 8.1 Create `src/services/pusher.rs` with `run_pusher_once(pool, config)` — shared by background loop and manual trigger
- [x] 8.2 Implement push record polling — calls `db::push_record::list_pending_records` + `db::push_record::list_retry_due_records`
- [x] 8.3 Implement channel/hotspot/keyword lookup for constructing push payload
- [x] 8.4 Implement webhook POST with `reqwest::Client` — construct DingTalk/Feishu text payload, handle response status
- [x] 8.5 Implement exponential backoff retry — `next_retry_at = now + retry_count * retry_base_seconds`, max retries from config
- [x] 8.6 Implement status update via `db::push_record::update_push_status` (success) and `update_push_status_optimistic` (failure with backoff)
- [x] 8.7 Implement `start_pusher_loop` — configurable interval ticker, calls `run_pusher_once`

## 9. Main.rs wiring

- [x] 9.1 Add `mod services;` to `src/main.rs`
- [x] 9.2 Implement mode-based background task spawning: `all` and `api` spawn parser + filter + pusher; `parser`/`filter`/`pusher` spawn single task + await ctrl-c
- [x] 9.3 Ensure background tasks receive cloned `pool` and `config` sub-struct

## 10. Verification

- [x] 10.1 `cargo check` — verify compilation with no errors
- [x] 10.2 `cargo run -- all` — verify server starts with all three background modules logging
- [x] 10.3 Test query APIs with curl: `/articles`, `/hotspots`, `/hotspots/{id}/push-records`, `/trend/{keyword_id}`
- [x] 10.4 Test trigger APIs with curl: `POST /trigger/filter`, `POST /trigger/pusher`
- [x] 10.5 Verify parser fetches RSS and inserts articles (add a test RSS source)
- [x] 10.6 Verify filter matches keywords and creates hot_events (check DB after manual trigger)
- [x] 10.7 Verify pusher sends webhooks (use a test webhook URL)
