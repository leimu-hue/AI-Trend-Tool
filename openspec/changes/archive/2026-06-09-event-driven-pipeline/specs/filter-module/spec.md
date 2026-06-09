## MODIFIED Requirements

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
