# source-crud-api

## Purpose

API endpoints for managing RSS data sources: list, create, update, delete, and manual fetch trigger. All endpoints require Bearer token authentication.

## Requirements

### Requirement: List sources endpoint

The system SHALL expose `GET /api/v1/sources` to list all data sources. Results SHALL be ordered by `created_at DESC`.

#### Scenario: List all sources

- **WHEN** `GET /api/v1/sources` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<DataSource>, ...]}`
- **THEN** each item SHALL contain all fields: `id`, `type`, `name`, `url`, `config`, `enabled`, `interval_seconds`, `last_fetched_at`, `created_at`, `updated_at`

#### Scenario: List sources without authentication

- **WHEN** `GET /api/v1/sources` is called without a valid Authorization header
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Create source endpoint

The system SHALL expose `POST /api/v1/sources` to add a new data source. The request body SHALL contain `type` (required), `name` (required), `url` (required), `interval_seconds` (optional, default 300), and `config` (optional, default "{}"). The response SHALL be HTTP 201 with the created `DataSource` object.

#### Scenario: Create source with required fields

- **WHEN** `POST /api/v1/sources` is called with body `{"type": "rss", "name": "Hacker News", "url": "https://hnrss.org/frontpage"}`
- **THEN** the system SHALL insert a row into `data_sources` with `interval_seconds = 300` and `config = "{}"`
- **THEN** the response SHALL be HTTP 201 with body `{"data": {"id": <id>, "type": "rss", "name": "Hacker News", ...}}`

#### Scenario: Create source with all fields

- **WHEN** `POST /api/v1/sources` is called with `interval_seconds` and `config` specified
- **THEN** the inserted row SHALL use the provided values instead of defaults

#### Scenario: Create source without authentication

- **WHEN** `POST /api/v1/sources` is called without a valid Bearer token
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Update source endpoint

The system SHALL expose `POST /api/v1/sources/{id}/update` to update an existing data source. All fields in the request body SHALL be optional — only provided fields are updated. The response SHALL be HTTP 200 with the updated `DataSource` object.

#### Scenario: Partial update

- **WHEN** `POST /api/v1/sources/1/update` is called with body `{"name": "Updated Name", "interval_seconds": 600}`
- **THEN** only `name` and `interval_seconds` SHALL be updated in the database
- **THEN** `updated_at` SHALL be set to the current timestamp
- **THEN** the response SHALL be HTTP 200 with the full updated `DataSource`

#### Scenario: Update non-existent source

- **WHEN** `POST /api/v1/sources/999/update` is called and no source with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Source 999 not found"}}`

#### Scenario: Update with empty body

- **WHEN** `POST /api/v1/sources/1/update` is called with body `{}`
- **THEN** no fields SHALL be modified
- **THEN** the response SHALL be HTTP 200 with the existing `DataSource`

### Requirement: Delete source endpoint

The system SHALL expose `POST /api/v1/sources/{id}/delete` to delete a data source. The response SHALL be HTTP 204 No Content on success.

#### Scenario: Delete existing source

- **WHEN** `POST /api/v1/sources/1/delete` is called with a valid Bearer token and source id=1 exists
- **THEN** the row SHALL be deleted from `data_sources`
- **THEN** the response SHALL be HTTP 204 with no body

#### Scenario: Delete non-existent source

- **WHEN** `POST /api/v1/sources/999/delete` is called and no source with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Source 999 not found"}}`

### Requirement: Manual fetch trigger endpoint

The system SHALL expose `POST /api/v1/sources/{id}/fetch` to trigger an immediate fetch for a specific source. It SHALL reset `last_fetched_at` to NULL so the Parser module picks it up on its next cycle. The response SHALL be HTTP 200 with a confirmation message.

#### Scenario: Trigger fetch for existing source

- **WHEN** `POST /api/v1/sources/1/fetch` is called with a valid Bearer token and source id=1 exists
- **THEN** the system SHALL set `last_fetched_at = NULL` for source id=1
- **THEN** the response SHALL be HTTP 200 with body `{"data": {"message": "Fetch triggered for source 1"}}`

#### Scenario: Trigger fetch for non-existent source

- **WHEN** `POST /api/v1/sources/999/fetch` is called and no source with id=999 exists
- **THEN** the response SHALL be HTTP 404

### Requirement: Source handlers follow project conventions

Source handlers SHALL reside at `src/handlers/source.rs` and be declared via `pub mod source;` in `src/handlers.rs`. All SQL operations SHALL delegate to `src/db/source.rs` functions. Responses SHALL use `ApiResponse` for consistent wrapping.

#### Scenario: Handler module compiles

- **WHEN** `cargo check` is run
- **THEN** `src/handlers/source.rs` SHALL compile as a submodule of `handlers`
- **THEN** no raw SQL strings SHALL appear in the handler file
