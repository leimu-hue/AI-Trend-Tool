# filter-module

## Purpose

Background service that matches unprocessed articles against enabled keywords using Aho-Corasick multi-pattern matching, accumulates hourly bucket counts, and detects trending hotspots via statistical burst detection (moving average + standard deviation).

## Requirements

### Requirement: Filter background scheduling

The system SHALL run the Filter module as an asynchronous background task on a configurable interval (default 300 seconds / 5 minutes) and expose a shared function for manual triggering.

#### Scenario: Filter runs on interval

- **WHEN** the Filter loop is running
- **THEN** it SHALL call `run_filter_once` every `config.filter.interval_seconds`

#### Scenario: Manual filter trigger calls same function

- **WHEN** the `POST /api/v1/trigger/filter` endpoint is called
- **THEN** it SHALL execute the same `run_filter_once` function that the background loop uses

### Requirement: Unprocessed article batching

The system SHALL fetch unprocessed articles in batches (size controlled by `config.filter.batch_size`) for keyword matching.

#### Scenario: Fetch unprocessed articles

- **WHEN** `run_filter_once` executes
- **THEN** it SHALL query `articles WHERE processed_at IS NULL ORDER BY fetched_at ASC LIMIT batch_size`

#### Scenario: No unprocessed articles — early return

- **WHEN** no unprocessed articles exist
- **THEN** the filter SHALL return immediately without error

### Requirement: Aho-Corasick keyword matching

The system SHALL build an Aho-Corasick automaton from all enabled keywords and match against each article's `title + summary` text.

#### Scenario: Build automaton from enabled keywords

- **WHEN** the filter runs
- **THEN** it SHALL load all keywords where `enabled = 1`
- **AND** SHALL build an Aho-Corasick automaton from their `word` values

#### Scenario: Case-insensitive matching

- **WHEN** a keyword has `case_sensitive = false`
- **THEN** the system SHALL match case-insensitively using `ascii_case_insensitive` mode

#### Scenario: Record keyword mentions

- **WHEN** a keyword matches in an article
- **THEN** the system SHALL insert a row into `keyword_mentions (keyword_id, article_id)`

#### Scenario: No enabled keywords — mark all processed

- **WHEN** no enabled keywords exist
- **THEN** the filter SHALL mark all unprocessed articles as processed and return

### Requirement: Hourly bucket counting

The system SHALL accumulate matched keyword counts per current UTC hour bucket (format `YYYYMMDDHH`).

#### Scenario: Accumulate counts per keyword per hour

- **WHEN** multiple articles match the same keyword in the same filter run
- **THEN** the system SHALL sum the match counts per keyword for the current hour bucket

### Requirement: Statistical burst detection

The system SHALL detect hotspots using a moving average + standard deviation model over historical hourly counts.

#### Scenario: Calculate statistics from historical counts

- **WHEN** a keyword has at least `config.filter.min_history_hours` of historical data in `hot_events`
- **THEN** the system SHALL calculate mean and standard deviation from the past `config.filter.history_hours` hourly counts

#### Scenario: Hotspot detected — count exceeds threshold

- **WHEN** `current_hour_count > mean + (keyword.std_multiplier * stddev)` AND `current_hour_count >= keyword.min_hot_count`
- **THEN** the system SHALL create a `hot_events` record and SHALL insert `push_records` (status='pending') for every enabled `push_channel`

#### Scenario: No hotspot — count within normal range

- **WHEN** `current_hour_count <= threshold` OR `current_hour_count < min_hot_count`
- **THEN** the system SHALL still record the hourly count in `hot_events` but SHALL NOT create push records

#### Scenario: Insufficient historical data

- **WHEN** a keyword has fewer than `min_history_hours` of historical hot_events
- **THEN** the system SHALL record the current count but SHALL NOT perform burst detection

### Requirement: Mark articles processed

The system SHALL mark all processed articles by setting `processed_at = datetime('now')` after matching completes.

#### Scenario: Batch update processed articles

- **WHEN** the filter completes matching for a batch
- **THEN** all articles in that batch SHALL have `processed_at` updated in chunks of 100
