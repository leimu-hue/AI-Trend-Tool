## MODIFIED Requirements

### Requirement: List sources endpoint

The system SHALL expose `GET /api/v1/sources` to list all data sources. Results SHALL be ordered by `created_at DESC`. Each item SHALL include `article_count` indicating the number of articles fetched from that source.

#### Scenario: List all sources

- **WHEN** `GET /api/v1/sources` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<SourceWithCount>, ...]}`
- **THEN** each item SHALL contain all fields: `id`, `type`, `name`, `url`, `config`, `enabled`, `interval_seconds`, `last_fetched_at`, `created_at`, `updated_at`, `article_count`

#### Scenario: List sources without authentication

- **WHEN** `GET /api/v1/sources` is called without a valid Authorization header
- **THEN** the request SHALL be rejected with HTTP 401
