# query-apis (delta)

## MODIFIED Requirements

### Requirement: Article list with pagination and filtering

文章列表 API `GET /api/v1/articles` SHALL 支持 `status` 查询参数（值: `pending`、`processing`、`matched`、`skipped`），替代 `processed` 参数。过渡期内 SHALL 同时支持 `processed` 参数（`processed=true` → `status=matched`，`processed=false` → `status=pending`）。

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

### Requirement: Push records for a hotspot

PushRecord 查询响应 SHALL 包含 `last_error` 字段。

#### Scenario: Push record response includes last_error

- **WHEN** client sends `GET /api/v1/hotspots/{id}/push-records` with valid Bearer token
- **THEN** each push record in the response SHALL include `"last_error"` field (string or null)
- **THEN** push records with `status='dead'` SHALL be included in the results
