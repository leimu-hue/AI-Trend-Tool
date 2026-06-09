## MODIFIED Requirements

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
