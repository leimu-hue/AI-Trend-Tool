# trigger-apis

## Purpose

Provide manual trigger endpoints for running the Filter and Pusher modules on demand, without waiting for their scheduled intervals.

## Requirements

### Requirement: Manual filter trigger

The system SHALL provide a `POST /api/v1/trigger/filter` endpoint that immediately runs one iteration of the Filter module and notifies the Pusher if new push records were created.

#### Scenario: Trigger filter successfully with downstream notification

- **WHEN** client sends `POST /api/v1/trigger/filter` with valid Bearer token
- **THEN** system executes `run_filter_once`
- **AND** if `run_filter_once` returns `true`, sends `PipelineEvent::NewData` via `push_ready_tx.try_send()`
- **AND** returns 200 with `{"data": {"message": "Filter executed"}}`

#### Scenario: Trigger filter without auth

- **WHEN** client sends `POST /api/v1/trigger/filter` without valid Bearer token
- **THEN** system returns 401 Unauthorized

### Requirement: Manual pusher trigger

The system SHALL provide a `POST /api/v1/trigger/pusher` endpoint that immediately runs one iteration of the Pusher module.

#### Scenario: Trigger pusher successfully

- **WHEN** client sends `POST /api/v1/trigger/pusher` with valid Bearer token
- **THEN** system executes `run_pusher_once` and returns 200 with `{"data": {"message": "Pusher executed"}}`

#### Scenario: Trigger pusher without auth

- **WHEN** client sends `POST /api/v1/trigger/pusher` without valid Bearer token
- **THEN** system returns 401 Unauthorized
