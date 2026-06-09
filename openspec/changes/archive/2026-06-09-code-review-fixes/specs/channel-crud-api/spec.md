## ADDED Requirements

### Requirement: Create channel validates input

The system SHALL validate input for `POST /api/v1/channels` before inserting into the database.

#### Scenario: Empty name rejected

- **WHEN** `POST /api/v1/channels` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "name must not be empty"}}`

#### Scenario: Empty config rejected

- **WHEN** `POST /api/v1/channels` body contains `config` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "config must not be empty"}}`

#### Scenario: Invalid JSON config rejected

- **WHEN** `POST /api/v1/channels` body contains `config` that is not valid JSON
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "config must be valid JSON"}}`

#### Scenario: Valid JSON config accepted

- **WHEN** `POST /api/v1/channels` is called with body `{"name": "DingTalk Alert", "config": "{\"url\": \"https://example.com/webhook\"}"}`
- **THEN** the channel SHALL be created normally

### Requirement: Update channel validates input

The system SHALL validate input for `POST /api/v1/channels/{id}/update` when fields are provided.

#### Scenario: Valid name accepted on update

- **WHEN** `POST /api/v1/channels/1/update` is called with `{"name": "New Name"}`
- **THEN** the update SHALL proceed normally

#### Scenario: Invalid JSON config rejected on update

- **WHEN** `POST /api/v1/channels/1/update` is called with `{"config": "not json"}`
- **THEN** the response SHALL be HTTP 400
