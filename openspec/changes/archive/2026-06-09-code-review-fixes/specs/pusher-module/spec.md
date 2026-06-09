## MODIFIED Requirements

### Requirement: Pending and retry-due record polling

The system SHALL query `push_records` for records with `status = 'pending'` or records with `status = 'failed'` that are due for retry. The retry count limit SHALL use the `max_retries` value from pusher config, passed as a parameter to the query function.

#### Scenario: Poll pending and retry-due records

- **WHEN** `run_pusher_once` executes
- **THEN** it SHALL fetch records where `status = 'pending'` OR (`status = 'failed'` AND `retry_count < config.pusher.max_retries` AND `next_retry_at <= now`)

#### Scenario: No pushable records — early return

- **WHEN** no pending or retry-due records exist
- **THEN** the pusher SHALL return immediately without error

### Requirement: Concurrent webhook push delivery

The system SHALL process pushable records concurrently with a maximum of 8 simultaneous webhook POST requests.

#### Scenario: Multiple records pushed concurrently

- **WHEN** multiple push records are pending
- **THEN** the system SHALL send webhook POST requests concurrently (max 8 at a time)
- **THEN** each push SHALL use optimistic locking to prevent duplicate sends

#### Scenario: Single record still works

- **WHEN** only one push record is pending
- **THEN** the system SHALL process it without error (concurrent stream of 1)

#### Scenario: Concurrent pushes share reqwest client

- **WHEN** concurrent pushes are in flight
- **THEN** all push tasks SHALL use clones of the same `reqwest::Client` instance
- **THEN** HTTP connection pooling SHALL be shared across all concurrent tasks
