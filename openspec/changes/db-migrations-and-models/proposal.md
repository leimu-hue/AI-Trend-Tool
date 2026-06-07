## Why

Step 01 delivered compilable project skeleton with empty models directory. No data layer exists — database tables, migrations, and Rust structs for querying data are missing. All subsequent steps (auth, CRUD APIs, background modules) require a defined schema and typed data models.

## What Changes

- Add `docs/migrations/<timestamp>_init.sql` — single DDL file with all 8 tables, indexes, and constraints
- Create `src/models/mod.rs` — module declarations for all 8 model files
- Create `src/models/token.rs` — `ApiToken` struct + `ApiTokenInfo` view DTO + `CreateTokenRequest`
- Create `src/models/source.rs` — `DataSource` struct + `CreateSourceRequest` + `UpdateSourceRequest`
- Create `src/models/article.rs` — `Article` struct + `ArticleQuery`
- Create `src/models/keyword.rs` — `Keyword` struct + `CreateKeywordRequest` + `UpdateKeywordRequest`
- Create `src/models/hot_event.rs` — `HotEvent` struct
- Create `src/models/channel.rs` — `PushChannel` struct + `CreateChannelRequest` + `UpdateChannelRequest`
- Create `src/models/push_record.rs` — `PushRecord` struct
- Wire migration runner into server startup so migrations auto-apply on `cargo run`

## Capabilities

### New Capabilities

- `database-schema`: 8 SQLite tables (api_tokens, data_sources, articles, keywords, keyword_mentions, hot_events, push_channels, push_records) with indexes, foreign key constraints with `ON DELETE CASCADE`, and default values
- `data-models`: Rust struct definitions for all 8 tables deriving `sqlx::FromRow` + `serde::Serialize`, plus request DTOs (`Create*Request`, `Update*Request`) and view DTOs (`ApiTokenInfo`, `ArticleQuery`) where needed

### Modified Capabilities

<!-- No existing specs changed — this is purely additive -->

## Impact

- **Affected code**: `src/main.rs` (migration runner call on startup), `src/models.rs` (module declarations), new `src/models/*.rs` files
- **Dependencies**: Already present in Cargo.toml — `sqlx` with `migrate` and `chrono` features, `serde`, `chrono`
- **Database**: Migration is additive only (all `CREATE TABLE IF NOT EXISTS`). No data loss risk on existing dev databases
- **Breaking changes**: None
