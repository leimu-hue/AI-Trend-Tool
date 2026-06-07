## 1. Data Source CRUD Handler

- [x] 1.1 Create `src/handlers/source.rs` with `list_sources` handler — GET, calls `db::source::list_sources`, wraps with `ApiResponse::ok`
- [x] 1.2 Add `create_source` handler — POST, calls `db::source::create_source`, returns 201 via `ApiResponse::created`
- [x] 1.3 Add `update_source` handler — POST `/{id}/update`, calls `db::source::get_source_by_id` then `db::source::update_source`, returns 404 if not found
- [x] 1.4 Add `delete_source` handler — POST `/{id}/delete`, calls `db::source::delete_source`, returns 204 or 404 if 0 rows affected
- [x] 1.5 Add `trigger_fetch` handler — POST `/{id}/fetch`, checks existence via `db::source::get_source_by_id`, resets `last_fetched_at` to NULL via new db function `db::source::reset_last_fetched`
- [x] 1.6 Add `db::source::reset_last_fetched(pool, id)` function that runs `UPDATE data_sources SET last_fetched_at = NULL WHERE id = ?`

## 2. Keyword CRUD Handler

- [x] 2.1 Create `src/handlers/keyword.rs` with `list_keywords` handler — GET, calls `db::keyword::list_keywords`, wraps with `ApiResponse::ok`
- [x] 2.2 Add `create_keyword` handler — POST, calls `db::keyword::create_keyword`, handles UNIQUE constraint → 409 Conflict
- [x] 2.3 Add `update_keyword` handler — POST `/{id}/update`, calls `db::keyword::get_keyword_by_id` then `db::keyword::update_keyword`, returns 404 if not found
- [x] 2.4 Add `delete_keyword` handler — POST `/{id}/delete`, calls `db::keyword::delete_keyword`, returns 204 or 404 if 0 rows affected

## 3. Push Channel CRUD Handler

- [x] 3.1 Create `src/handlers/channel.rs` with `list_channels` handler — GET, calls `db::channel::list_channels`, wraps with `ApiResponse::ok`
- [x] 3.2 Add `create_channel` handler — POST, calls `db::channel::create_channel`, returns 201 via `ApiResponse::created`
- [x] 3.3 Add `update_channel` handler — POST `/{id}/update`, calls `db::channel::get_channel_by_id` then `db::channel::update_channel`, returns 404 if not found
- [x] 3.4 Add `delete_channel` handler — POST `/{id}/delete`, calls `db::channel::delete_channel`, returns 204 or 404 if 0 rows affected

## 4. Module Wiring & Routes

- [x] 4.1 Update `src/handlers.rs` — uncomment `pub mod source;`, `pub mod keyword;`, `pub mod channel;`
- [x] 4.2 Update `src/routes.rs` — register all 13 routes in `api_routes()` using POST-only convention:
  - Sources: GET `/sources`, POST `/sources`, POST `/sources/{id}/update`, POST `/sources/{id}/delete`, POST `/sources/{id}/fetch`
  - Keywords: GET `/keywords`, POST `/keywords`, POST `/keywords/{id}/update`, POST `/keywords/{id}/delete`
  - Channels: GET `/channels`, POST `/channels`, POST `/channels/{id}/update`, POST `/channels/{id}/delete`

## 5. Build & Verify

- [x] 5.1 Run `cargo check` — confirm zero compilation errors
- [x] 5.2 Run `cargo build` — confirm successful build
- [x] 5.3 Start server with `cargo run -- --config config.toml api` and manually test endpoints using curl
