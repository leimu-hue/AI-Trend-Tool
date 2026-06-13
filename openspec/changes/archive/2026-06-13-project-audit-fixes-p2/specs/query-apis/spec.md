## MODIFIED Requirements

### Requirement: Keyword trend data

The system SHALL provide a `GET /api/v1/trend/{keyword_id}` endpoint that returns hourly count data points for a keyword using data from the `hot_events` table. The `hours` parameter SHALL be clamped to range 1..8760 (1 year max) and safely cast to i32.

#### Scenario: Get trend for existing keyword with default hours

- **WHEN** client sends `GET /api/v1/trend/3` with valid Bearer token and keyword 3 exists
- **THEN** system returns 200 with `{"data": {"keyword_id": 3, "keyword": "<word>", "points": [...]}}` for last 24 hours

#### Scenario: Hours parameter clamped to safe range

- **WHEN** client sends `GET /api/v1/trend/3?hours=2147483648` (exceeds i32::MAX)
- **THEN** system SHALL clamp hours to 8760 (1 year max) instead of truncating
- **THEN** the query SHALL execute successfully without negative hour values

#### Scenario: Get trend for non-existent keyword

- **WHEN** client sends `GET /api/v1/trend/99999` with valid Bearer token and keyword does not exist
- **THEN** system returns 404 Not Found
