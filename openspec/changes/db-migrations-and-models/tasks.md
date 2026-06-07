## 1. Database Migration

- [x] 1.1 Create `docs/migrations/` directory if not present and run `sqlx migrate add init` to scaffold migration file
- [x] 1.2 Write complete DDL into the generated migration file: all 8 `CREATE TABLE IF NOT EXISTS` statements with columns, indexes, foreign keys, and UNIQUE constraints per the plan

## 2. Rust Data Models — Module Entry

- [x] 2.1 Replace `src/models/.gitkeep` with `src/models.rs` containing `pub mod` declarations for all 8 model files (token, source, article, keyword, hot_event, channel, push_record)

## 3. Rust Data Models — Entity Structs

- [x] 3.1 Create `src/models/token.rs` — `ApiToken` (FromRow, Serialize), `ApiTokenInfo` (Serialize, From<ApiToken>), `CreateTokenRequest` (Deserialize)
- [x] 3.2 Create `src/models/source.rs` — `DataSource` (FromRow, Serialize, Deserialize), `CreateSourceRequest` (Deserialize), `UpdateSourceRequest` (Deserialize)
- [x] 3.3 Create `src/models/article.rs` — `Article` (FromRow, Serialize), `ArticleQuery` (Deserialize)
- [x] 3.4 Create `src/models/keyword.rs` — `Keyword` (FromRow, Serialize), `CreateKeywordRequest` (Deserialize), `UpdateKeywordRequest` (Deserialize)
- [x] 3.5 Create `src/models/hot_event.rs` — `HotEvent` (FromRow, Serialize)
- [x] 3.6 Create `src/models/channel.rs` — `PushChannel` (FromRow, Serialize), `CreateChannelRequest` (Deserialize), `UpdateChannelRequest` (Deserialize)
- [x] 3.7 Create `src/models/push_record.rs` — `PushRecord` (FromRow, Serialize)

## 4. Migration Runner Integration

- [x] 4.1 Add `sqlx::migrate!()` call in `src/main.rs` startup sequence — run migrations against the connection pool before starting the HTTP server

## 5. Verification

- [x] 5.1 Run `cargo check` to verify all models compile and migration SQL is embedded without errors
- [x] 5.2 Run `cargo run -- --config config.toml api` to verify migrations apply cleanly and the server starts
- [x] 5.3 Run `sqlite3 <db_path> ".tables"` to confirm all 8 tables exist in the created database
