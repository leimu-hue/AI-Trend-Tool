# parser-module

## Purpose

Background service that periodically scans enabled RSS data sources, fetches their feeds, parses articles, and stores new articles in the database with link-based deduplication.

## Requirements

### Requirement: Parser background scheduling

The system SHALL run the Parser module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and pipeline events. The polling interval SHALL be configurable via `config.toml` (`parser.interval_seconds`).

#### Scenario: Parser scans on configurable interval

- **WHEN** the Parser loop is running
- **THEN** every `config.parser.interval_seconds` seconds it SHALL query `data_sources` for enabled sources where `NOW() - last_fetched_at >= interval_seconds` (or `last_fetched_at IS NULL`)

#### Scenario: Parser respects concurrent fetch limit

- **WHEN** multiple data sources need fetching simultaneously
- **THEN** the Parser SHALL limit concurrent fetches to `config.parser.max_concurrent_fetches` using a semaphore

#### Scenario: Parser notifies Filter on new articles

- **WHEN** the Parser successfully inserts one or more articles in a fetch cycle
- **THEN** it SHALL send `PipelineEvent::NewData` via `articles_ready_tx.try_send()`

#### Scenario: Parser shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Parser SHALL log a shutdown message and break out of its loop

### Requirement: RSS feed parsing

The system SHALL parse RSS/Atom feeds using the `feed-rs` library and extract article entries.

#### Scenario: Parse RSS feed

- **WHEN** a data source with `type = "rss"` is fetched
- **THEN** the system SHALL extract `link`, `title`, `summary` from each feed entry
- **AND** SHALL use `published` or `updated` date when available

#### Scenario: Article deduplication by link

- **WHEN** an extracted article has a `link` that already exists in the `articles` table
- **THEN** the system SHALL skip that article (no duplicate row inserted)

#### Scenario: Update last_fetched_at on success

- **WHEN** a data source is successfully fetched and parsed
- **THEN** the system SHALL update `data_sources.last_fetched_at` to the current timestamp

#### Scenario: Log error on fetch failure

- **WHEN** a data source fetch fails (network error, invalid feed, etc.)
- **THEN** the system SHALL log the error and continue to the next source (no crash)

### Requirement: Extensible parser trait

The system SHALL define a `Parser` trait using the `async-trait` crate to enable future parser type additions.

#### Scenario: Trait method signature

- **WHEN** a new parser type implements the `Parser` trait
- **THEN** it SHALL provide `async fn fetch_and_parse(&self, source: &DataSource) -> Result<Vec<ParsedArticle>, Box<dyn Error + Send + Sync>>`

#### Scenario: RssParser implements Parser trait

- **WHEN** the Parser loop processes an RSS data source
- **THEN** it SHALL dispatch through the `Parser` trait interface
