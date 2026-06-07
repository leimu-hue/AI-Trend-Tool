## ADDED Requirements

### Requirement: Article list with pagination and filtering

The system SHALL provide a `GET /api/v1/articles` endpoint that returns paginated articles with optional filtering by source and processed status.

#### Scenario: List all articles with default pagination

- **WHEN** client sends `GET /api/v1/articles` with valid Bearer token
- **THEN** system returns 200 with `{"data": {"items": [...], "total": <N>, "page": 1, "per_page": 20}}`

#### Scenario: Filter articles by source

- **WHEN** client sends `GET /api/v1/articles?source_id=3` with valid Bearer token
- **THEN** system returns 200 with only articles where `source_id = 3`

#### Scenario: Filter unprocessed articles

- **WHEN** client sends `GET /api/v1/articles?processed=false` with valid Bearer token
- **THEN** system returns 200 with only articles where `processed_at IS NULL`

#### Scenario: Paginate beyond first page

- **WHEN** client sends `GET /api/v1/articles?page=3&per_page=10` with valid Bearer token
- **THEN** system returns 200 with the 3rd page of 10 items each

#### Scenario: Per page limit cap

- **WHEN** client sends `GET /api/v1/articles?per_page=500` with valid Bearer token
- **THEN** system treats `per_page` as 100 (maximum)

#### Scenario: Unauthenticated request

- **WHEN** client sends `GET /api/v1/articles` without valid Bearer token
- **THEN** system returns 401 Unauthorized

### Requirement: Hotspot event list with pagination

The system SHALL provide a `GET /api/v1/hotspots` endpoint that returns paginated hotspot events with optional keyword filter.

#### Scenario: List all hotspots

- **WHEN** client sends `GET /api/v1/hotspots` with valid Bearer token
- **THEN** system returns 200 with paginated hotspot events ordered by `created_at DESC`

#### Scenario: Filter hotspots by keyword

- **WHEN** client sends `GET /api/v1/hotspots?keyword_id=5` with valid Bearer token
- **THEN** system returns 200 with only hotspots for keyword ID 5

### Requirement: Push records for a hotspot

The system SHALL provide a `GET /api/v1/hotspots/{id}/push-records` endpoint that returns all push records for a given hotspot event.

#### Scenario: Get push records for existing hotspot

- **WHEN** client sends `GET /api/v1/hotspots/42/push-records` with valid Bearer token and hotspot 42 exists
- **THEN** system returns 200 with array of push records for that hotspot, ordered by channel

#### Scenario: Get push records for non-existent hotspot

- **WHEN** client sends `GET /api/v1/hotspots/99999/push-records` with valid Bearer token and hotspot does not exist
- **THEN** system returns 200 with empty array

### Requirement: Keyword trend data

The system SHALL provide a `GET /api/v1/trend/{keyword_id}` endpoint that returns hourly count data points for a keyword using data from the `hot_events` table.

#### Scenario: Get trend for existing keyword with default hours

- **WHEN** client sends `GET /api/v1/trend/3` with valid Bearer token and keyword 3 exists
- **THEN** system returns 200 with `{"data": {"keyword_id": 3, "keyword": "<word>", "points": [{"hour_bucket": "...", "count": N}, ...]}}` for last 24 hours

#### Scenario: Get trend with custom hour range

- **WHEN** client sends `GET /api/v1/trend/3?hours=48` with valid Bearer token
- **THEN** system returns 200 with points covering the last 48 hours

#### Scenario: Get trend for non-existent keyword

- **WHEN** client sends `GET /api/v1/trend/99999` with valid Bearer token and keyword does not exist
- **THEN** system returns 404 Not Found
