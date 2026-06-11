## MODIFIED Requirements

### Requirement: Parser background scheduling

The system SHALL run the Parser module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and pipeline events. The system SHALL use `tokio::task::JoinSet` to manage spawned fetch sub-tasks. On shutdown, the system SHALL wait for all in-flight fetch tasks to complete before exiting.

#### Scenario: Parser scans on configurable interval

- **WHEN** the Parser loop is running
- **THEN** every `config.parser.interval_seconds` seconds it SHALL query `data_sources` for enabled sources where `NOW() - last_fetched_at >= interval_seconds` (or `last_fetched_at IS NULL`)

#### Scenario: Parser notifies Filter on new articles

- **WHEN** the Parser successfully inserts one or more articles in a fetch cycle
- **THEN** it SHALL send `PipelineEvent::NewData` via `articles_ready_tx.try_send()`

#### Scenario: Parser waits for child tasks on shutdown

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Parser SHALL log a shutdown message
- **THEN** the Parser SHALL await all in-flight fetch tasks via `JoinSet::join_next()`
- **THEN** the Parser SHALL break out of its loop after all tasks complete
