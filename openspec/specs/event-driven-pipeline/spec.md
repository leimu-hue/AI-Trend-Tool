# event-driven-pipeline

## Purpose

In-process event bus connecting Parser → Filter → Pusher via `tokio::sync::mpsc` channels for near-real-time downstream notification, with configurable interval fallback polling.

## Requirements

### Requirement: Pipeline event bus initialization

The system SHALL create a `Pipeline` struct at startup that holds inter-module communication channels (`articles_ready_tx`, `push_ready_tx`) and a shared `CancellationToken`.

#### Scenario: Pipeline created with both channels

- **WHEN** `Pipeline::new()` is called
- **THEN** it SHALL return a `Pipeline` with `articles_ready_tx` and `push_ready_tx` senders
- **AND** SHALL return two receivers (`articles_ready_rx`, `push_ready_rx`) for downstream modules
- **AND** SHALL create a shared `CancellationToken`

#### Scenario: Pipeline is cloneable

- **WHEN** `Pipeline::clone()` is called
- **THEN** the clone SHALL share the same underlying channel senders and cancellation token

### Requirement: Pipeline event signaling

The system SHALL define a `PipelineEvent::NewData` variant and send it through mpsc channels when modules produce new data.

#### Scenario: Parser signals new articles to Filter

- **WHEN** the Parser successfully inserts one or more new articles
- **THEN** it SHALL call `pipeline.articles_ready_tx.try_send(PipelineEvent::NewData)` to notify Filter

#### Scenario: Filter signals new push records to Pusher

- **WHEN** `run_filter_once` creates one or more new push records
- **THEN** the caller SHALL call `pipeline.push_ready_tx.try_send(PipelineEvent::NewData)` to notify Pusher

#### Scenario: Channel full — non-blocking send

- **WHEN** a module calls `try_send` on a full channel (16 pending signals)
- **THEN** `try_send` SHALL return `Err(TrySendError::Full)` without blocking
- **AND** the sender SHALL ignore the error and continue (interval fallback guarantees eventual processing)
