## MODIFIED Requirements

### Requirement: Pusher background scheduling

The system SHALL run the Pusher module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and `push_ready_rx` channel (event-driven trigger from Filter). The system SHALL expose a shared `run_pusher_once` function for manual triggering.

#### Scenario: Pusher runs on interval

- **WHEN** the Pusher loop is running and the interval ticks
- **THEN** it SHALL call `run_pusher_once`

#### Scenario: Pusher runs on Filter notification

- **WHEN** the Pusher loop receives `PipelineEvent::NewData` via `push_rx.recv()`
- **THEN** it SHALL immediately call `run_pusher_once`

#### Scenario: Manual pusher trigger calls same function

- **WHEN** the `POST /api/v1/trigger/pusher` endpoint is called
- **THEN** it SHALL execute the same `run_pusher_once` function that the background loop uses

#### Scenario: Pusher shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Pusher SHALL log a shutdown message and break out of its loop
