# keyword-crud-api

## Purpose

API endpoints for managing hotspot keywords: list, create, update, and delete. All endpoints require Bearer token authentication.

## Requirements

### Requirement: List keywords endpoint

The system SHALL expose `GET /api/v1/keywords` to list all keywords. Results SHALL be ordered by `created_at DESC`.

#### Scenario: List all keywords

- **WHEN** `GET /api/v1/keywords` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<Keyword>, ...]}`
- **THEN** each item SHALL contain all fields: `id`, `word`, `case_sensitive`, `enabled`, `std_multiplier`, `min_hot_count`, `created_at`

#### Scenario: List keywords without authentication

- **WHEN** `GET /api/v1/keywords` is called without a valid Authorization header
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Create keyword endpoint

The system SHALL expose `POST /api/v1/keywords` to add a new keyword. The request body SHALL contain `word` (required), `case_sensitive` (optional, default false), `std_multiplier` (optional, default 2.0), and `min_hot_count` (optional, default 3). The response SHALL be HTTP 201 with the created `Keyword` object.

#### Scenario: Create keyword with required field only

- **WHEN** `POST /api/v1/keywords` is called with body `{"word": "GPT-5"}`
- **THEN** the system SHALL insert a row into `keywords` with `case_sensitive = false`, `std_multiplier = 2.0`, `min_hot_count = 3`
- **THEN** the response SHALL be HTTP 201 with body `{"data": {"id": <id>, "word": "GPT-5", ...}}`

#### Scenario: Create keyword with all fields

- **WHEN** `POST /api/v1/keywords` is called with `case_sensitive`, `std_multiplier`, and `min_hot_count` specified
- **THEN** the inserted row SHALL use the provided values instead of defaults

#### Scenario: Create duplicate keyword

- **WHEN** `POST /api/v1/keywords` is called with a `word` that already exists
- **THEN** the response SHALL be HTTP 409 with body `{"error": {"code": "CONFLICT", "message": "Keyword '<word>' already exists"}}`

#### Scenario: Create keyword without authentication

- **WHEN** `POST /api/v1/keywords` is called without a valid Bearer token
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Update keyword endpoint

The system SHALL expose `POST /api/v1/keywords/{id}/update` to update an existing keyword. All fields in the request body SHALL be optional — only provided fields are updated. The response SHALL be HTTP 200 with the updated `Keyword` object.

#### Scenario: Partial update

- **WHEN** `POST /api/v1/keywords/1/update` is called with body `{"std_multiplier": 3.0, "enabled": false}`
- **THEN** only `std_multiplier` and `enabled` SHALL be updated in the database
- **THEN** the response SHALL be HTTP 200 with the full updated `Keyword`

#### Scenario: Update non-existent keyword

- **WHEN** `POST /api/v1/keywords/999/update` is called and no keyword with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Keyword 999 not found"}}`

#### Scenario: Update with empty body

- **WHEN** `POST /api/v1/keywords/1/update` is called with body `{}`
- **THEN** no fields SHALL be modified
- **THEN** the response SHALL be HTTP 200 with the existing `Keyword`

### Requirement: Delete keyword endpoint

The system SHALL expose `POST /api/v1/keywords/{id}/delete` to delete a keyword. The response SHALL be HTTP 204 No Content on success.

#### Scenario: Delete existing keyword

- **WHEN** `POST /api/v1/keywords/1/delete` is called with a valid Bearer token and keyword id=1 exists
- **THEN** the row SHALL be deleted from `keywords`
- **THEN** the response SHALL be HTTP 204 with no body

#### Scenario: Delete non-existent keyword

- **WHEN** `POST /api/v1/keywords/999/delete` is called and no keyword with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Keyword 999 not found"}}`

### Requirement: Keyword handlers follow project conventions

Keyword handlers SHALL reside at `src/handlers/keyword.rs` and be declared via `pub mod keyword;` in `src/handlers.rs`. All SQL operations SHALL delegate to `src/db/keyword.rs` functions. Responses SHALL use `ApiResponse` for consistent wrapping.

#### Scenario: Handler module compiles

- **WHEN** `cargo check` is run
- **THEN** `src/handlers/keyword.rs` SHALL compile as a submodule of `handlers`
- **THEN** no raw SQL strings SHALL appear in the handler file
