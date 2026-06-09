# filter-module

## Purpose

Background service that matches unprocessed articles against enabled keywords using Aho-Corasick multi-pattern matching, accumulates hourly bucket counts, and detects trending hotspots via statistical burst detection (moving average + standard deviation).

## Requirements

### Requirement: Filter background scheduling

The system SHALL run the Filter module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and `articles_ready_rx` channel (event-driven trigger from Parser). The system SHALL expose a shared `run_filter_once` function that returns `bool` indicating whether new push records were created.

#### Scenario: Filter runs on interval

- **WHEN** the Filter loop is running and the interval ticks
- **THEN** it SHALL call `run_filter_once` and if it returns `true`, send `PipelineEvent::NewData` via `push_ready_tx.try_send()`

#### Scenario: Filter runs on Parser notification

- **WHEN** the Filter loop receives `PipelineEvent::NewData` via `articles_rx.recv()`
- **THEN** it SHALL immediately call `run_filter_once` and if it returns `true`, send `PipelineEvent::NewData` via `push_ready_tx.try_send()`

#### Scenario: run_filter_once returns true when push records created

- **WHEN** `run_filter_once` executes and burst detection identifies a hotspot, creating one or more push records
- **THEN** the function SHALL return `true`

#### Scenario: run_filter_once returns false when no new push records

- **WHEN** `run_filter_once` executes and no hotspots are detected (or no unprocessed articles exist)
- **THEN** the function SHALL return `false`

#### Scenario: Filter shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Filter SHALL log a shutdown message and break out of its loop

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

#### Scenario: Record keyword mentions in batch

- **WHEN** a keyword matches in an article
- **THEN** the system SHALL accumulate the (keyword_id, article_id) pair for batch insertion

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

### Requirement: Keyword mention recording uses batch insert

The system SHALL collect all keyword-article mention pairs during the matching phase and insert them in a single batch operation using `INSERT OR IGNORE INTO keyword_mentions (keyword_id, article_id) VALUES (?, ?), (?, ?), ...` in chunks of 100.

#### Scenario: Batch insert keyword mentions

- **WHEN** multiple keywords match in multiple articles during one filter run
- **THEN** the system SHALL collect all (keyword_id, article_id) pairs into a Vec
- **THEN** SHALL insert them in chunks of 100 using a single INSERT with multiple VALUES
- **THEN** SHALL use `INSERT OR IGNORE` to preserve deduplication semantics

#### Scenario: No matches — skip batch insert

- **WHEN** no keywords match any article in the batch
- **THEN** the system SHALL NOT execute any INSERT for keyword mentions

### Requirement: Hot event upsert uses ON CONFLICT

The system SHALL upsert hot_event records using SQLite's `ON CONFLICT(keyword_id, hour_bucket) DO UPDATE` syntax to preserve the row's `id` and maintain foreign key integrity with `push_records`.

#### Scenario: First hotspot for keyword in hour — insert

- **WHEN** a hotspot is detected and no existing row matches the (keyword_id, hour_bucket) pair
- **THEN** the system SHALL INSERT a new `hot_events` row
- **THEN** push_records SHALL be created referencing the new row's id

#### Scenario: Repeat detection in same hour — update

- **WHEN** a hotspot is re-detected for the same (keyword_id, hour_bucket) pair
- **THEN** the system SHALL UPDATE the existing row's `count`, `mean_historical`, and `stddev_historical`
- **THEN** the row's `id` SHALL NOT change
- **THEN** existing push_records referencing this hot_event SHALL remain valid

### Requirement: Historical statistics use batch query

The system SHALL load all keywords' hourly counts in a single query rather than per-keyword queries. Statistics (mean, stddev) SHALL be calculated in memory from the batched result.

#### Scenario: Batch load all hourly counts

- **WHEN** `run_filter_once` executes hotspot detection
- **THEN** the system SHALL query all (keyword_id, hour_bucket, total_count) in one SQL call
- **THEN** SHALL group results by keyword_id in memory
- **THEN** SHALL compute mean and stddev per keyword from the grouped data

#### Scenario: Keyword with no history skipped

- **WHEN** a keyword has zero rows in the batched result
- **THEN** the system SHALL skip burst detection for that keyword (insufficient data)

### Requirement: Mark articles processed

The system SHALL mark all processed articles by setting `processed_at = datetime('now')` after matching completes.

#### Scenario: Batch update processed articles

- **WHEN** the filter completes matching for a batch
- **THEN** all articles in that batch SHALL have `processed_at` updated in chunks of 100
