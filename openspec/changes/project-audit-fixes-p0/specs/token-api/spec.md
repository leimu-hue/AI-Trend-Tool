## MODIFIED Requirements

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
