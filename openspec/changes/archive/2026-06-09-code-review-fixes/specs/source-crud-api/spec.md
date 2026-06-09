## ADDED Requirements

### Requirement: Create source validates input

The system SHALL validate input for `POST /api/v1/sources` before inserting into the database.

#### Scenario: Empty name rejected

- **WHEN** `POST /api/v1/sources` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "name must not be empty"}}`

#### Scenario: Empty URL rejected

- **WHEN** `POST /api/v1/sources` body contains `url` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "url must not be empty"}}`

#### Scenario: Invalid URL scheme rejected

- **WHEN** `POST /api/v1/sources` body contains `url` not starting with `http://` or `https://`
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "url must start with http:// or https://"}}`

### Requirement: Update source validates input

The system SHALL validate input for `POST /api/v1/sources/{id}/update` when fields are provided.

#### Scenario: Non-empty name accepted

- **WHEN** `POST /api/v1/sources/1/update` is called with `{"name": "Valid Name"}`
- **THEN** the update SHALL proceed normally

#### Scenario: Whitespace-only name rejected

- **WHEN** `POST /api/v1/sources/1/update` is called with `{"name": "   "}`
- **THEN** the response SHALL be HTTP 400

#### Scenario: Invalid URL scheme rejected on update

- **WHEN** `POST /api/v1/sources/1/update` is called with `{"url": "ftp://invalid.com"}`
- **THEN** the response SHALL be HTTP 400
