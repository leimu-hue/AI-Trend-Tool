## MODIFIED Requirements

### Requirement: Pusher background scheduling

The system SHALL run the Pusher module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and `push_ready_rx` channel (event-driven trigger from Filter). A single `reqwest::Client` SHALL be created in `start_pusher_loop` and passed as a reference to each `run_pusher_once` call. The system SHALL expose a shared `run_pusher_once` function for manual triggering.

#### Scenario: Pusher runs on interval

- **WHEN** the Pusher loop is running and the interval ticks
- **THEN** it SHALL call `run_pusher_once` with the shared `reqwest::Client` reference

#### Scenario: Pusher runs on Filter notification

- **WHEN** the Pusher loop receives `PipelineEvent::NewData` via `push_rx.recv()`
- **THEN** it SHALL immediately call `run_pusher_once` with the shared `reqwest::Client` reference

#### Scenario: Manual pusher trigger calls same function

- **WHEN** the `POST /api/v1/trigger/pusher` endpoint is called
- **THEN** it SHALL create a temporary `reqwest::Client` and call `run_pusher_once`

#### Scenario: Pusher shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Pusher SHALL log a shutdown message and break out of its loop

### Requirement: Pending and retry-due record polling

The system SHALL first atomically claim push_records by updating status from 'pending' to 'processing', then query for records with `status = 'processing'` for webhook delivery. The retry count limit SHALL use the `max_retries` value from pusher config.

#### Scenario: Atomically claim pending records

- **WHEN** `run_pusher_once` executes
- **THEN** it SHALL first execute `UPDATE push_records SET status='processing' WHERE status='pending' AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))`
- **THEN** it SHALL then SELECT records WHERE `status='processing'` for webhook delivery

#### Scenario: No pushable records — early return

- **WHEN** no pending or retry-due records exist
- **THEN** the pusher SHALL return immediately without error
