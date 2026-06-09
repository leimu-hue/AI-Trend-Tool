## MODIFIED Requirements

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
