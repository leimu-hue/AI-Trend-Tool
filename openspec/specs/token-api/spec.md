# token-api

## Purpose

API endpoints for managing API tokens: create, list, and revoke. All endpoints require authentication via Bearer token.

## Requirements

### Requirement: Create token endpoint

The system SHALL expose `POST /api/v1/tokens` to create a new API token. The request body SHALL contain `name` (required, must not be empty or whitespace-only) and `expires_at` (optional). The system SHALL generate a 64-character random hex token, compute its SHA-256 hash, insert `***REDACTED***` into the `token` column and the SHA-256 hash into `token_hash`, and return HTTP 201 with the full `ApiToken` object where the `token` field contains the plaintext (set in memory after INSERT).

#### Scenario: Create token with name only

- **WHEN** `POST /api/v1/tokens` is called with body `{"name": "Frontend UI"}`
- **THEN** the system SHALL generate a 64-character hex token string
- **THEN** the system SHALL compute `token_hash = SHA256(token)`
- **THEN** the system SHALL insert a row into `api_tokens` with `token = '***REDACTED***'` and `token_hash = <sha256>`
- **THEN** the system SHALL set the `token` field of the returned `ApiToken` object to the plaintext token in memory
- **THEN** the response SHALL be HTTP 201 with body `{"data": {"id": <id>, "name": "Frontend UI", "token": "<generated_hex>", "token_hash": "<sha256>", "created_at": "<iso8601>", "expires_at": null, "revoked": false}}`

#### Scenario: Create token with empty name

- **WHEN** `POST /api/v1/tokens` is called with body `{"name": "   "}`
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "name must not be empty"}}`

#### Scenario: Create token with expiry

- **WHEN** `POST /api/v1/tokens` is called with body `{"name": "Temp Token", "expires_at": "2025-12-31T23:59:59"}`
- **THEN** the inserted row SHALL have `token = '***REDACTED***'` and `expires_at = 2025-12-31T23:59:59`
- **THEN** the response SHALL include `"expires_at": "2025-12-31T23:59:59"` and `"token": "<plaintext>"`

#### Scenario: Create token without authentication

- **WHEN** `POST /api/v1/tokens` is called without a valid Authorization header
- **THEN** the request SHALL be rejected by the auth middleware with HTTP 401 before reaching the handler

### Requirement: Auth middleware validates token via hash

The system SHALL authenticate API requests by computing SHA-256 of the presented Bearer token and looking up `token_hash` in `api_tokens`.

#### Scenario: Valid token authenticates via hash lookup

- **WHEN** a request presents a valid Bearer token
- **THEN** the auth middleware SHALL compute `SHA256(token)`
- **THEN** SHALL query `api_tokens WHERE token_hash = ? AND revoked = 0`
- **THEN** SHALL check expiry and allow the request through on success

#### Scenario: Revoked token cannot authenticate

- **WHEN** a token has been revoked via `POST /api/v1/tokens/{id}/revoke`
- **THEN** subsequent requests using that token SHALL receive HTTP 401 `"Invalid or revoked token"`

### Requirement: Revoke token endpoint

The system SHALL expose `POST /api/v1/tokens/{id}/revoke` to revoke a token. The system SHALL first verify the token exists, then update `revoked = 1`. The response SHALL be HTTP 204 No Content on success.

#### Scenario: Revoke existing token

- **WHEN** `POST /api/v1/tokens/2/revoke` is called with a valid Bearer token and token id=2 exists
- **THEN** the system SHALL first check that the token exists
- **THEN** the system SHALL set `revoked = 1` for the row with `id = 2`
- **THEN** the response SHALL be HTTP 204 with no body

#### Scenario: Revoke non-existent token

- **WHEN** `POST /api/v1/tokens/999/revoke` is called and no token with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Token with id 999 not found"}}`

### Requirement: List tokens endpoint

The system SHALL expose `GET /api/v1/tokens` to list all tokens. The response SHALL use `ApiTokenInfo` which omits the `token` field for security. Tokens SHALL be ordered by `created_at DESC`.

#### Scenario: List all tokens

- **WHEN** `GET /api/v1/tokens` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<ApiTokenInfo>, ...]}`
- **THEN** each item SHALL contain `id`, `name`, `token_hash`, `last_used_at`, `created_at`, `expires_at`, `revoked`
- **THEN** the `token` field SHALL NOT appear in any item

#### Scenario: Tokens ordered newest first

- **WHEN** multiple tokens exist with different `created_at` values
- **THEN** the response array SHALL have the most recently created token first

### Requirement: Token handlers follow 2018 edition module style

The token handler implementations SHALL reside at `src/handlers/token.rs` and be declared via `pub mod token;` in `src/handlers.rs`. No `src/handlers/mod.rs` file SHALL exist.

#### Scenario: Handler module compiles

- **WHEN** `cargo check` is run
- **THEN** `src/handlers/token.rs` SHALL compile as a submodule of `handlers`
- **THEN** `src/handlers/mod.rs` SHALL NOT exist

### Requirement: Tokens 页面使用 IPC clipboard API

前端 Tokens 管理页面的复制功能 SHALL 使用 preload 脚本暴露的 `window.electronAPI.clipboard.writeText()` IPC 桥接，而不是已废弃的 `document.execCommand('copy')`。

#### Scenario: 复制 token 到剪贴板
- **WHEN** 用户在 Tokens 页面点击复制按钮
- **THEN** 系统 SHALL 调用 `window.electronAPI.clipboard.writeText(tokenString)`
- **THEN** 系统 SHALL NOT 使用 `document.execCommand('copy')`
- **THEN** 复制成功 SHALL 显示提示信息
