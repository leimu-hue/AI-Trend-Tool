## MODIFIED Requirements

### Requirement: Article list with pagination and filtering

The system SHALL provide a `GET /api/v1/articles` endpoint that returns paginated articles with optional filtering by source and processed status. The `per_page` value in the response SHALL reflect the actual limit applied (clamped to max 100), not the raw user input.

#### Scenario: List all articles with default pagination

- **WHEN** client sends `GET /api/v1/articles` with valid Bearer token
- **THEN** system returns 200 with `{"data": {"items": [...], "total": <N>, "page": 1, "per_page": 20}}`

#### Scenario: Paginate beyond first page

- **WHEN** client sends `GET /api/v1/articles?page=3&per_page=10` with valid Bearer token
- **THEN** system returns 200 with the 3rd page of 10 items each
- **THEN** response `per_page` SHALL be 10

#### Scenario: Per page limit cap

- **WHEN** client sends `GET /api/v1/articles?per_page=500` with valid Bearer token
- **THEN** system treats `per_page` as 100 (maximum)
- **THEN** response `per_page` SHALL be 100, not 500

### Requirement: Hotspot event list with pagination

The system SHALL provide a `GET /api/v1/hotspots` endpoint that returns paginated hotspot events with optional keyword filter. The `per_page` value in the response SHALL reflect the actual limit applied.

#### Scenario: Per page cap reflected in response

- **WHEN** client sends `GET /api/v1/hotspots?per_page=200` with valid Bearer token
- **THEN** response `per_page` SHALL be 100, reflecting the clamp
