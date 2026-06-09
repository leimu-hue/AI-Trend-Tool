## ADDED Requirements

### Requirement: api_tokens table has token_hash column

The system SHALL have a `token_hash` column in `api_tokens` storing the SHA-256 hash of the token value. The column SHALL be `TEXT NOT NULL DEFAULT ''`. A UNIQUE index SHALL exist on `token_hash`.

#### Scenario: New token gets hash stored

- **WHEN** a new API token is created
- **THEN** `token_hash` SHALL be set to `SHA256(token)`
- **THEN** the UNIQUE index on `token_hash` SHALL prevent duplicate hash values

#### Scenario: Existing tokens preserve backward compatibility

- **WHEN** the migration is applied to an existing database
- **THEN** existing rows SHALL have `token_hash = ''` (empty default)
- **THEN** the `token` column SHALL remain unchanged

## MODIFIED Requirements

### Requirement: hot_events table

The system SHALL have a `hot_events` table storing detected hotspot events with hourly bucket statistics.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `keyword_id` (INTEGER NOT NULL FK→keywords ON DELETE CASCADE), `hour_bucket` (TEXT NOT NULL — format YYYYMMDDHH), `count` (INTEGER NOT NULL DEFAULT 0), `mean_historical` (REAL NOT NULL DEFAULT 0.0), `stddev_historical` (REAL NOT NULL DEFAULT 0.0), `created_at` (DATETIME NOT NULL DEFAULT current time).

**Constraints**: `UNIQUE(keyword_id, hour_bucket)` — one record per keyword per hour bucket.

**Indexes**: `idx_hot_events_keyword` ON `keyword_id`, `idx_hot_events_bucket` ON `hour_bucket`.

#### Scenario: Detect a hotspot

- **WHEN** a hotspot event is upserted with keyword_id, hour_bucket, count, mean, and stddev
- **THEN** the row SHALL be persisted with `created_at` set to current time
- **THEN** the bucket index SHALL enable efficient time-range queries

#### Scenario: Duplicate keyword_id + hour_bucket updates existing row

- **WHEN** a hotspot event is upserted with a (keyword_id, hour_bucket) pair that already exists
- **THEN** the existing row SHALL be updated with new count, mean_historical, and stddev_historical values
- **THEN** the row's `id` SHALL remain unchanged, preserving foreign key references from `push_records`
