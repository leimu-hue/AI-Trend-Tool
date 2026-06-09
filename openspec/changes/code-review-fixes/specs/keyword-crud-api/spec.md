## ADDED Requirements

### Requirement: Create keyword validates input

The system SHALL validate input for `POST /api/v1/keywords` before inserting into the database.

#### Scenario: Empty word rejected

- **WHEN** `POST /api/v1/keywords` body contains `word` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "word must not be empty"}}`

#### Scenario: Valid word accepted

- **WHEN** `POST /api/v1/keywords` is called with body `{"word": "GPT-5"}`
- **THEN** the keyword SHALL be created normally

#### Scenario: Non-positive std_multiplier rejected

- **WHEN** `POST /api/v1/keywords` body contains `std_multiplier` as 0 or negative
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "std_multiplier must be positive"}}`

#### Scenario: min_hot_count below 1 rejected

- **WHEN** `POST /api/v1/keywords` body contains `min_hot_count` as 0 or negative
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "min_hot_count must be >= 1"}}`

### Requirement: Update keyword validates input

The system SHALL validate input for `POST /api/v1/keywords/{id}/update` when fields are provided.

#### Scenario: Valid std_multiplier on update accepted

- **WHEN** `POST /api/v1/keywords/1/update` is called with `{"std_multiplier": 2.5}`
- **THEN** the update SHALL proceed normally

#### Scenario: Non-positive std_multiplier on update rejected

- **WHEN** `POST /api/v1/keywords/1/update` is called with `{"std_multiplier": 0}`
- **THEN** the response SHALL be HTTP 400

#### Scenario: min_hot_count below 1 on update rejected

- **WHEN** `POST /api/v1/keywords/1/update` is called with `{"min_hot_count": 0}`
- **THEN** the response SHALL be HTTP 400
