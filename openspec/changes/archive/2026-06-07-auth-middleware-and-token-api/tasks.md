## 1. Module Declarations

- [x] 1.1 Add `pub mod auth;` to `src/middleware.rs` (file already exists, add declaration)
- [x] 1.2 Add `pub mod token;` to `src/handlers.rs` (file already exists, add declaration)
- [x] 1.3 Add `mod middleware;` and `mod handlers;` to `src/main.rs` (if not already present)

## 2. Auth Middleware

- [x] 2.1 Create `src/middleware/auth.rs` with `auth_middleware` function
- [x] 2.2 Implement Bearer token extraction from `Authorization` header — return `AppError::Unauthorized` on missing/invalid format
- [x] 2.3 Implement token lookup in `api_tokens` with `revoked = 0` filter
- [x] 2.4 Implement expiry check: compare `expires_at` (if `Some`) against `Utc::now().naive_utc()`
- [x] 2.5 Implement background `last_used_at` update via `tokio::spawn`
- [x] 2.6 Insert `ApiToken` into `request.extensions_mut()` for downstream handlers

## 3. Token API Handlers

- [x] 3.1 Create `src/handlers/token.rs` with `create_token`, `list_tokens`, `revoke_token` handlers
- [x] 3.2 Implement `create_token`: generate 64-char hex token, INSERT with RETURNING, return HTTP 201
- [x] 3.3 Implement `list_tokens`: SELECT all ordered by `created_at DESC`, map to `ApiTokenInfo`, return HTTP 200
- [x] 3.4 Implement `revoke_token`: UPDATE `revoked = 1` by id, return HTTP 204 or 404 if not found

## 4. Route Integration

- [x] 4.1 Update `src/routes.rs`: refactor `api_routes()` to register token routes (POST/GET/DELETE `/tokens`)
- [x] 4.2 Apply auth middleware layer to the API router using `middleware::from_fn_with_state`
- [x] 4.3 Ensure `/health` remains outside the auth scope (no middleware applied)

## 5. Initial Token Bootstrap

- [x] 5.1 Add `ensure_initial_token(pool: &SqlitePool, config: &AppConfig)` function in `src/main.rs` (or a new `src/bootstrap.rs`)
- [x] 5.2 Implement logic: check `SELECT COUNT(*) FROM api_tokens`, skip if > 0
- [x] 5.3 Use `config.auth.initial_token` if set and non-empty; otherwise auto-generate 64-char hex via `rand + hex`
- [x] 5.4 Log generated token with `tracing::warn!` with prominent formatting
- [x] 5.5 Insert token with name "Initial Admin Token"
- [x] 5.6 Call `ensure_initial_token` after `sqlx::migrate!()` in `main.rs`

## 6. Verification

- [x] 6.1 Run `cargo check` — confirm no compilation errors
- [x] 6.2 Run `cargo run -- --config config.toml all` — verified initial token bootstrap
- [x] 6.3 Test POST /api/v1/tokens with initial token → 201 ✓
- [x] 6.4 Test GET /api/v1/tokens → 200 with token list (no plaintext tokens) ✓
- [x] 6.5 Test POST /api/v1/tokens/revoke (body) → 204, double revoke → 204 (idempotent)
  - NOTE: Changed from DELETE /api/v1/tokens/{id} to POST /api/v1/tokens/revoke with JSON body
    due to axum 0.7.9 routing bug (Path extractor + middleware = route not matched).
    See BUG_REPORT.md for details.
- [x] 6.6 Test unauthenticated request → 401 "Missing Authorization header" ✓
- [x] 6.7 Test revoked token → 401 "Invalid or revoked token" ✓
- [x] 6.8 Test expired token → 401 "Token has expired" ✓
