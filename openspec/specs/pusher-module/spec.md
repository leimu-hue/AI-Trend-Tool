# pusher-module

## Purpose

Background service that polls pending and retry-due push records, sends webhook POST requests to configured push channels (DingTalk/Feishu), and manages retry with exponential backoff and optimistic locking.

## Requirements

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

### Requirement: Pending and retry-due record polling

The system SHALL query `push_records` for records with `status = 'pending'` or records with `status = 'failed'` that are due for retry.

#### Scenario: Poll pending and retry-due records

- **WHEN** `run_pusher_once` executes
- **THEN** it SHALL fetch records where `status = 'pending'` OR (`status = 'failed'` AND `retry_count < max_retries` AND `next_retry_at <= now`)

#### Scenario: No pushable records — early return

- **WHEN** no pending or retry-due records exist
- **THEN** the pusher SHALL return immediately without error

### Requirement: Webhook push delivery

The system SHALL send a POST request to the channel's webhook URL with a JSON payload describing the hotspot event.

#### Scenario: Send webhook for pending record

- **WHEN** a push record is pending
- **THEN** the system SHALL fetch the associated `push_channel`, `hot_event`, and `keyword`
- **AND** SHALL POST a JSON payload to the channel's webhook URL
- **AND** SHALL include the keyword word and hotspot count in the message

#### Scenario: Mark success on HTTP 2xx

- **WHEN** the webhook POST returns a 2xx status
- **THEN** the system SHALL update the push record status to `success`

#### Scenario: Mark failed on non-2xx response

- **WHEN** the webhook POST returns a non-2xx status
- **THEN** the system SHALL increment `retry_count` and set status to `failed` with exponential backoff for `next_retry_at`

#### Scenario: Mark failed on network error

- **WHEN** the webhook POST fails with a network error
- **THEN** the system SHALL increment `retry_count` and set status to `failed` with exponential backoff

#### Scenario: Skip channel with no webhook URL

- **WHEN** a push channel's config JSON does not contain a valid `url` field
- **THEN** the system SHALL mark the push record as `failed` and log an error

### Requirement: Exponential backoff retry

The system SHALL implement exponential backoff for failed push attempts: `next_retry_at = now + (retry_count * retry_base_seconds)`.

#### Scenario: First retry scheduled

- **WHEN** a push fails and `retry_count` becomes 1
- **THEN** `next_retry_at` SHALL be set to `now + retry_base_seconds`

#### Scenario: Second retry with longer backoff

- **WHEN** a push fails and `retry_count` becomes 2
- **THEN** `next_retry_at` SHALL be set to `now + 2 * retry_base_seconds`

#### Scenario: Max retries exhausted

- **WHEN** `retry_count` reaches `config.pusher.max_retries`
- **THEN** `next_retry_at` SHALL be set to NULL (no further retries)

### Requirement: Optimistic locking for push records

The system SHALL use optimistic locking when updating push record status to prevent duplicate sends from concurrent pusher runs.

#### Scenario: Optimistic update succeeds

- **WHEN** the pusher updates a record from `pending` to a new status
- **THEN** it SHALL update only if current status matches expected status

#### Scenario: Optimistic update skipped

- **WHEN** another process already changed the record's status
- **THEN** the update SHALL affect zero rows and the pusher SHALL skip that record
