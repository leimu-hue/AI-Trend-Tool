# database-schema

## Purpose

Defines the SQLite database schema for TrendAITool — 8 tables covering API tokens, RSS data sources, fetched articles, keywords, keyword-article mentions, hotspot events, push notification channels, and push delivery records. All tables include indexes, foreign key constraints with ON DELETE CASCADE, and appropriate default values.

## Requirements

### Requirement: Migration auto-runs on startup

The system SHALL automatically apply all pending SQLite migrations on server startup via `sqlx::migrate!()`. Migrations SHALL run before any HTTP server or background module starts. Failed migrations SHALL cause the process to exit with an error.

#### Scenario: Fresh database on first startup

- **WHEN** the server starts and no database file exists at the configured path
- **THEN** the system SHALL create the database file and apply all migrations
- **THEN** all 8 tables SHALL exist with their specified columns, indexes, and constraints
- **THEN** the server SHALL proceed to start normally

#### Scenario: Existing database with all migrations applied

- **WHEN** the server starts and the database already has all migrations applied
- **THEN** the system SHALL detect no pending migrations and skip the migration step
- **THEN** existing data SHALL remain intact
- **THEN** the server SHALL proceed to start normally

#### Scenario: Migration file has SQL error

- **WHEN** a migration file contains invalid SQL syntax
- **THEN** `cargo build` SHALL fail due to the compile-time migration embedding
- **THEN** the error message SHALL include the offending SQL statement

### Requirement: api_tokens table

The system SHALL have an `api_tokens` table storing bearer tokens for API authentication.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `name` (TEXT NOT NULL), `token` (TEXT NOT NULL, stores placeholder `***REDACTED***`), `token_hash` (TEXT NOT NULL DEFAULT ''), `last_used_at` (DATETIME), `created_at` (DATETIME NOT NULL DEFAULT current time), `expires_at` (DATETIME), `revoked` (BOOLEAN NOT NULL DEFAULT 0).

**Indexes**: UNIQUE index on `token_hash`.

#### Scenario: Create a new API token

- **WHEN** a row is inserted into `api_tokens` with a name
- **THEN** the row SHALL be persisted with `token = '***REDACTED***'`, `token_hash = SHA256(plaintext)`
- **THEN** `created_at` SHALL be set to the current time
- **THEN** `revoked` SHALL default to `0` (false)
- **THEN** `id` SHALL be auto-assigned

#### Scenario: Duplicate token hash value

- **WHEN** a row is inserted with a `token_hash` value that already exists
- **THEN** the insert SHALL fail with a UNIQUE constraint violation

#### Scenario: New token gets hash stored

- **WHEN** a new API token is created
- **THEN** `token_hash` SHALL be set to `SHA256(token)`
- **THEN** `token` SHALL be set to `***REDACTED***`
- **THEN** the UNIQUE index on `token_hash` SHALL prevent duplicate hash values

#### Scenario: Existing tokens preserve backward compatibility

- **WHEN** the migration is applied to an existing database
- **THEN** existing rows SHALL have `token` updated to `'***REDACTED***'`
- **THEN** `token_hash` SHALL remain unchanged

### Requirement: data_sources table

The system SHALL have a `data_sources` table storing RSS/Atom feed configurations.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `type` (TEXT NOT NULL — rss/atom/json_feed), `name` (TEXT NOT NULL), `url` (TEXT NOT NULL), `config` (TEXT NOT NULL DEFAULT '{}' — JSON extension), `enabled` (BOOLEAN NOT NULL DEFAULT 1), `interval_seconds` (INTEGER NOT NULL DEFAULT 300), `last_fetched_at` (DATETIME), `created_at` (DATETIME NOT NULL DEFAULT current time), `updated_at` (DATETIME NOT NULL DEFAULT current time).

#### Scenario: Add an RSS data source

- **WHEN** a new data source is inserted with type "rss", a name, and a URL
- **THEN** the row SHALL be persisted with `enabled = 1`, `interval_seconds = 300`, `config = '{}'`
- **THEN** `created_at` and `updated_at` SHALL be set to the current time

### Requirement: articles table

The system SHALL have an `articles` table storing fetched articles.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `source_id` (INTEGER NOT NULL FK→data_sources ON DELETE CASCADE), `link` (TEXT NOT NULL UNIQUE — dedup key), `title` (TEXT NOT NULL DEFAULT ''), `summary` (TEXT NOT NULL DEFAULT ''), `content` (TEXT NOT NULL DEFAULT ''), `published_at` (DATETIME), `fetched_at` (DATETIME NOT NULL DEFAULT current time), `processed_at` (DATETIME — NULL = unprocessed).

**Indexes**: `idx_articles_processed` ON `processed_at`, `idx_articles_source` ON `source_id`, `idx_articles_fetched` ON `fetched_at`.

#### Scenario: Fetch new article

- **WHEN** a new article is inserted with a unique link
- **THEN** the row SHALL be persisted with `fetched_at` set to current time
- **THEN** `processed_at` SHALL be NULL

#### Scenario: Duplicate article link

- **WHEN** an article is inserted with a `link` that already exists
- **THEN** the insert SHALL fail with a UNIQUE constraint violation
- **THEN** deduplication is enforced at the database level

#### Scenario: Source deleted with cascade

- **WHEN** a data source row is deleted
- **THEN** all articles with `source_id` referencing that source SHALL also be deleted

### Requirement: keywords table

The system SHALL have a `keywords` table storing monitored keywords with sensitivity parameters for hotspot detection.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `word` (TEXT NOT NULL UNIQUE), `case_sensitive` (BOOLEAN NOT NULL DEFAULT 0), `enabled` (BOOLEAN NOT NULL DEFAULT 1), `std_multiplier` (REAL NOT NULL DEFAULT 2.0), `min_hot_count` (INTEGER NOT NULL DEFAULT 3), `created_at` (DATETIME NOT NULL DEFAULT current time).

#### Scenario: Add a keyword with default sensitivity

- **WHEN** a keyword is inserted with only the word field specified
- **THEN** `std_multiplier` SHALL default to 2.0
- **THEN** `min_hot_count` SHALL default to 3
- **THEN** `case_sensitive` SHALL default to 0 (false)

#### Scenario: Add duplicate keyword

- **WHEN** a keyword is inserted with a `word` that already exists
- **THEN** the insert SHALL fail with a UNIQUE constraint violation

### Requirement: keyword_mentions table

The system SHALL have a `keyword_mentions` table recording each keyword-article match event.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `keyword_id` (INTEGER NOT NULL FK→keywords ON DELETE CASCADE), `article_id` (INTEGER NOT NULL FK→articles ON DELETE CASCADE), `matched_at` (DATETIME NOT NULL DEFAULT current time).

**Indexes**: `idx_mentions_keyword` ON `keyword_id`, `idx_mentions_article` ON `article_id`, `idx_mentions_unique` UNIQUE ON `(keyword_id, article_id)`.

#### Scenario: Record a keyword match

- **WHEN** a keyword match is recorded with keyword_id and article_id
- **THEN** the row SHALL be persisted with `matched_at` set to current time
- **THEN** the indexes SHALL optimize lookups by keyword and by article

#### Scenario: Duplicate (keyword_id, article_id) ignored

- **WHEN** a keyword match is recorded for a (keyword_id, article_id) pair that already exists
- **THEN** `INSERT OR IGNORE` SHALL silently skip the duplicate

#### Scenario: Keyword or article deleted with cascade

- **WHEN** a keyword row is deleted
- **THEN** all mention rows with `keyword_id` referencing that keyword SHALL also be deleted
- **WHEN** an article row is deleted
- **THEN** all mention rows with `article_id` referencing that article SHALL also be deleted

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

### Requirement: push_channels table

The system SHALL have a `push_channels` table storing alert notification channel configurations.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `name` (TEXT NOT NULL), `channel_type` (TEXT NOT NULL DEFAULT 'webhook'), `config` (TEXT NOT NULL DEFAULT '{}' — JSON with webhook URL), `enabled` (BOOLEAN NOT NULL DEFAULT 1).

#### Scenario: Add a DingTalk webhook channel

- **WHEN** a push channel is inserted with name, channel_type="webhook", and config='{"url": "https://oapi.dingtalk.com/robot/send?access_token=xxx"}'
- **THEN** the row SHALL be persisted with `enabled = 1`

### Requirement: push_records table

The system SHALL have a `push_records` table tracking per-hotspot per-channel push delivery status.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `hot_event_id` (INTEGER NOT NULL FK→hot_events ON DELETE CASCADE), `channel_id` (INTEGER NOT NULL FK→push_channels ON DELETE CASCADE), `status` (TEXT NOT NULL DEFAULT 'pending' — pending/success/failed), `retry_count` (INTEGER NOT NULL DEFAULT 0), `next_retry_at` (DATETIME), `created_at` (DATETIME NOT NULL DEFAULT current time), `updated_at` (DATETIME NOT NULL DEFAULT current time).

**Constraints**: `UNIQUE(hot_event_id, channel_id)` — one record per hotspot per channel.

**Indexes**: `idx_push_records_status` ON `status`.

#### Scenario: Create push record for a hotspot

- **WHEN** a hotspot is detected and push records are created for each enabled channel
- **THEN** each record SHALL have `status = 'pending'`, `retry_count = 0`
- **THEN** duplicate (hot_event_id, channel_id) pairs SHALL be rejected by the UNIQUE constraint

#### Scenario: Pusher polls pending records

- **WHEN** the pusher queries `push_records WHERE status = 'pending'`
- **THEN** the status index SHALL provide efficient lookup
- **THEN** only records matching the status SHALL be returned
