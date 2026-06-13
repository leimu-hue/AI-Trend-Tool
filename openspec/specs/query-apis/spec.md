# query-apis

## Purpose

Provide read APIs for fetched articles, detected hotspots, push records, and keyword trend data. All endpoints use paginated responses with optional filtering.

## Requirements

### Requirement: Article list with pagination and filtering

文章列表 API `GET /api/v1/articles` SHALL 支持 `status` 查询参数（值: `pending`、`processing`、`matched`、`skipped`），替代 `processed` 参数。过渡期内 SHALL 同时支持 `processed` 参数（`processed=true` → `status=matched`，`processed=false` → `status=pending`）。

#### Scenario: List all articles with default pagination

- **WHEN** client sends `GET /api/v1/articles` with valid Bearer token
- **THEN** system returns 200 with `{"data": {"items": [...], "total": <N>, "page": 1, "per_page": 20}}`

#### Scenario: Filter articles by source

- **WHEN** client sends `GET /api/v1/articles?source_id=3` with valid Bearer token
- **THEN** system returns 200 with only articles where `source_id = 3`

#### Scenario: Filter articles by status

- **WHEN** client sends `GET /api/v1/articles?status=matched` with valid Bearer token
- **THEN** system returns 200 with only articles where `status = 'matched'`

#### Scenario: Filter pending articles via status

- **WHEN** client sends `GET /api/v1/articles?status=pending` with valid Bearer token
- **THEN** system returns 200 with only articles where `status = 'pending'`

#### Scenario: Legacy processed parameter still works

- **WHEN** client sends `GET /api/v1/articles?processed=false` with valid Bearer token
- **THEN** system SHALL internally map to `status='pending'`
- **THEN** system returns 200 with only pending articles

#### Scenario: Legacy processed=true parameter still works

- **WHEN** client sends `GET /api/v1/articles?processed=true` with valid Bearer token
- **THEN** system SHALL internally map to `status='matched'`
- **THEN** system returns 200 with only matched articles

#### Scenario: status and processed both provided — status wins

- **WHEN** client sends `GET /api/v1/articles?processed=true&status=skipped` with valid Bearer token
- **THEN** system SHALL use `status=skipped`（status 参数优先级高于 processed）

#### Scenario: Invalid status value rejected

- **WHEN** client sends `GET /api/v1/articles?status=invalid` with valid Bearer token
- **THEN** system returns 400 Bad Request with error code `INVALID_STATUS`

#### Scenario: Paginate beyond first page

- **WHEN** client sends `GET /api/v1/articles?page=3&per_page=10` with valid Bearer token
- **THEN** system returns 200 with the 3rd page of 10 items each

#### Scenario: Per page limit cap

- **WHEN** client sends `GET /api/v1/articles?per_page=500` with valid Bearer token
- **THEN** system treats `per_page` as 100 (maximum)
- **THEN** response `per_page` SHALL be 100, not 500

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

#### Scenario: Per page cap reflected in response

- **WHEN** client sends `GET /api/v1/hotspots?per_page=200` with valid Bearer token
- **THEN** response `per_page` SHALL be 100, reflecting the clamp

### Requirement: Push records for a hotspot

The system SHALL provide a `GET /api/v1/hotspots/{id}/push-records` endpoint that returns all push records for a given hotspot event. PushRecord 查询响应 SHALL 包含 `last_error` 字段。

#### Scenario: Get push records for existing hotspot

- **WHEN** client sends `GET /api/v1/hotspots/42/push-records` with valid Bearer token and hotspot 42 exists
- **THEN** system returns 200 with array of push records for that hotspot, ordered by channel
- **THEN** each push record in the response SHALL include `"last_error"` field (string or null)
- **THEN** push records with `status='dead'` SHALL be included in the results

#### Scenario: Get push records for non-existent hotspot

- **WHEN** client sends `GET /api/v1/hotspots/99999/push-records` with valid Bearer token and hotspot does not exist
- **THEN** system returns 200 with empty array

### Requirement: Keyword trend data

The system SHALL provide a `GET /api/v1/trend/{keyword_id}` endpoint that returns hourly count data points for a keyword using data from the `hot_events` table. The `hours` parameter SHALL be clamped to range 1..8760 (1 year max) and safely cast to i32.

#### Scenario: Get trend for existing keyword with default hours

- **WHEN** client sends `GET /api/v1/trend/3` with valid Bearer token and keyword 3 exists
- **THEN** system returns 200 with `{"data": {"keyword_id": 3, "keyword": "<word>", "points": [{"hour_bucket": "...", "count": N}, ...]}}` for last 24 hours

#### Scenario: Get trend with custom hour range

- **WHEN** client sends `GET /api/v1/trend/3?hours=48` with valid Bearer token
- **THEN** system returns 200 with points covering the last 48 hours

#### Scenario: Hours parameter clamped to safe range

- **WHEN** client sends `GET /api/v1/trend/3?hours=2147483648` (exceeds i32::MAX)
- **THEN** system SHALL clamp hours to 8760 (1 year max) instead of truncating
- **THEN** the query SHALL execute successfully without negative hour values

#### Scenario: Get trend for non-existent keyword

- **WHEN** client sends `GET /api/v1/trend/99999` with valid Bearer token and keyword does not exist
- **THEN** system returns 404 Not Found
