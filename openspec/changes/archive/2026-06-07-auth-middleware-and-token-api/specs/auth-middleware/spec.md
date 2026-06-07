## ADDED Requirements

### Requirement: Bearer token extraction

The auth middleware SHALL extract a Bearer token from the `Authorization` header of incoming requests. If the header is missing or does not use the `Bearer` scheme, the middleware SHALL return HTTP 401 with error code `UNAUTHORIZED`.

#### Scenario: Missing Authorization header

- **WHEN** a request arrives at `/api/v1/*` without an `Authorization` header
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Missing Authorization header"}}`

#### Scenario: Non-Bearer Authorization header

- **WHEN** a request arrives with `Authorization: Basic dXNlcjpwYXNz`
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Invalid Authorization format, expected Bearer"}}`

### Requirement: Token validation against database

The auth middleware SHALL query the `api_tokens` table for the extracted token string where `revoked = 0`. If no matching token exists, the middleware SHALL return HTTP 401.

#### Scenario: Valid token passes through

- **WHEN** a request arrives with `Authorization: Bearer <valid_token>` and the token exists in `api_tokens` with `revoked = 0`
- **THEN** the middleware SHALL call `next.run(request).await` to pass the request to the next handler

#### Scenario: Invalid or revoked token

- **WHEN** a request arrives with a token that does not exist in `api_tokens` OR has `revoked = 1`
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Invalid or revoked token"}}`

### Requirement: Token expiry check

If the token has an `expires_at` value, the middleware SHALL compare it against the current UTC time. If the token has expired, the middleware SHALL return HTTP 401.

#### Scenario: Expired token

- **WHEN** a request arrives with a valid but expired token (`expires_at < Utc::now()`)
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Token has expired"}}`

#### Scenario: Non-expiring token passes

- **WHEN** a request arrives with a valid token where `expires_at IS NULL`
- **THEN** the middleware SHALL skip the expiry check and pass the request through

### Requirement: Last used timestamp update

The auth middleware SHALL update the token's `last_used_at` field to the current timestamp on each successful authentication. This update SHALL run in a background `tokio::spawn` task to avoid blocking the response.

#### Scenario: last_used_at updated asynchronously

- **WHEN** a request is successfully authenticated with a valid Bearer token
- **THEN** a background task SHALL execute `UPDATE api_tokens SET last_used_at = datetime('now') WHERE id = ?`
- **THEN** the response SHALL NOT be delayed by this update

### Requirement: Token injection into request extensions

After successful authentication, the middleware SHALL insert the full `ApiToken` struct into the request's extensions, making it available to downstream handlers via `request.extensions().get::<ApiToken>()`.

#### Scenario: Handler reads authenticated token

- **WHEN** a handler receives a request that passed the auth middleware
- **THEN** `request.extensions().get::<ApiToken>()` SHALL return `Some(&ApiToken)` with all fields populated

### Requirement: Health endpoint excluded from auth

The `/health` endpoint SHALL NOT require authentication. Only routes under `/api/v1/*` SHALL be protected by the auth middleware.

#### Scenario: Health check accessible without token

- **WHEN** `GET /health` is called without any Authorization header
- **THEN** the server SHALL return HTTP 200 with `{"status": "ok"}`

### Requirement: Auth middleware follows 2018 edition module style

The auth middleware implementation SHALL reside at `src/middleware/auth.rs` and be declared via `pub mod auth;` in `src/middleware.rs`. No `src/middleware/mod.rs` file SHALL exist.

#### Scenario: Module structure is correct

- **WHEN** `cargo check` is run
- **THEN** `src/middleware/auth.rs` SHALL compile as a submodule of `middleware`
- **THEN** `src/middleware/mod.rs` SHALL NOT exist
