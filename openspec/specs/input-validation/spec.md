# input-validation

## Purpose

Input validation for all mutation handler endpoints, rejecting empty strings, invalid URLs, and malformed JSON before they reach the database layer.

## Requirements

### Requirement: Source handler input validation

The system SHALL validate input for `POST /api/v1/sources` and `POST /api/v1/sources/{id}/update` handlers using `validator` crate derive macros on request structs, returning 400 Bad Request for invalid input.

#### Scenario: Empty name rejected for create source

- **WHEN** `POST /api/v1/sources` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "<validation error>"}}`

#### Scenario: Invalid URL rejected for create source

- **WHEN** `POST /api/v1/sources` body contains `url` not parseable as a valid URL
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "<validation error>"}}`

### Requirement: Keyword handler input validation

The system SHALL validate input for `POST /api/v1/keywords` and `POST /api/v1/keywords/{id}/update` handlers using `validator` crate derive macros.

#### Scenario: Empty word rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `word` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "word must not be empty"}}`

#### Scenario: Non-positive std_multiplier rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `std_multiplier` <= 0
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "std_multiplier must be positive"}}`

#### Scenario: min_hot_count below 1 rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `min_hot_count` < 1
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "min_hot_count must be >= 1"}}`

### Requirement: Channel handler input validation

The system SHALL validate input for `POST /api/v1/channels` and `POST /api/v1/channels/{id}/update` handlers, returning 400 Bad Request for invalid input.

#### Scenario: Empty name rejected for create channel

- **WHEN** `POST /api/v1/channels` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "name must not be empty"}}`

#### Scenario: Empty config rejected for create channel

- **WHEN** `POST /api/v1/channels` body contains `config` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "config must not be empty"}}`

#### Scenario: Invalid JSON config rejected for create channel

- **WHEN** `POST /api/v1/channels` body contains `config` that is not valid JSON
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "config must be valid JSON"}}`

### Requirement: Token handler input validation

The system SHALL validate input for `POST /api/v1/tokens` handler using `validator` crate derive macros.

#### Scenario: Empty name rejected for create token

- **WHEN** `POST /api/v1/tokens` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "name must not be empty"}}`
