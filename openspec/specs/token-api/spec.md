# token-api

## Purpose

API endpoints for managing API tokens: create, list, and revoke. All endpoints require authentication via Bearer token.

## Requirements

### Requirement: Create token endpoint

The system SHALL expose `POST /api/v1/tokens` to create a new API token. The request body SHALL contain `name` (required) and `expires_at` (optional). The system SHALL generate a 64-character random hex token, insert into `api_tokens`, and return HTTP 201 with the full `ApiToken` (including the plaintext token).

#### Scenario: Create token with name only

- **WHEN** `POST /api/v1/tokens` is called with body `{"name": "Frontend UI"}`
- **THEN** the system SHALL generate a 64-character hex token string
- **THEN** the system SHALL insert a row into `api_tokens` with `name = "Frontend UI"`, `expires_at = NULL`
- **THEN** the response SHALL be HTTP 201 with body `{"data": {"id": <id>, "name": "Frontend UI", "token": "<generated_hex>", "created_at": "<iso8601>", "expires_at": null, "revoked": false}}`

#### Scenario: Create token with expiry

- **WHEN** `POST /api/v1/tokens` is called with body `{"name": "Temp Token", "expires_at": "2025-12-31T23:59:59"}`
- **THEN** the inserted row SHALL have `expires_at = 2025-12-31T23:59:59`
- **THEN** the response SHALL include `"expires_at": "2025-12-31T23:59:59"`

#### Scenario: Create token without authentication

- **WHEN** `POST /api/v1/tokens` is called without a valid Authorization header
- **THEN** the request SHALL be rejected by the auth middleware with HTTP 401 before reaching the handler

### Requirement: List tokens endpoint

The system SHALL expose `GET /api/v1/tokens` to list all tokens. The response SHALL use `ApiTokenInfo` which omits the `token` field for security. Tokens SHALL be ordered by `created_at DESC`.

#### Scenario: List all tokens

- **WHEN** `GET /api/v1/tokens` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<ApiTokenInfo>, ...]}`
- **THEN** each item SHALL contain `id`, `name`, `last_used_at`, `created_at`, `expires_at`, `revoked`
- **THEN** the `token` field SHALL NOT appear in any item

#### Scenario: Tokens ordered newest first

- **WHEN** multiple tokens exist with different `created_at` values
- **THEN** the response array SHALL have the most recently created token first

### Requirement: Revoke token endpoint

The system SHALL expose `DELETE /api/v1/tokens/{id}` to revoke a token. Revocation SHALL be a soft delete: `UPDATE api_tokens SET revoked = 1 WHERE id = ?`. The response SHALL be HTTP 204 No Content.

#### Scenario: Revoke existing token

- **WHEN** `DELETE /api/v1/tokens/2` is called with a valid Bearer token and token id=2 exists
- **THEN** the system SHALL set `revoked = 1` for the row with `id = 2`
- **THEN** the response SHALL be HTTP 204 with no body

#### Scenario: Revoke non-existent token

- **WHEN** `DELETE /api/v1/tokens/999` is called and no token with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Token with id 999 not found"}}`

#### Scenario: Revoked token cannot authenticate

- **WHEN** a token has been revoked via `DELETE /api/v1/tokens/{id}`
- **THEN** subsequent requests using that token SHALL receive HTTP 401 `"Invalid or revoked token"`

### Requirement: Token handlers follow 2018 edition module style

The token handler implementations SHALL reside at `src/handlers/token.rs` and be declared via `pub mod token;` in `src/handlers.rs`. No `src/handlers/mod.rs` file SHALL exist.

#### Scenario: Handler module compiles

- **WHEN** `cargo check` is run
- **THEN** `src/handlers/token.rs` SHALL compile as a submodule of `handlers`
- **THEN** `src/handlers/mod.rs` SHALL NOT exist
